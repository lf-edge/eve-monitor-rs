use std::collections::HashMap;

use super::{
    events::{EFIVariableBootEvent, GrubEvent},
    tpmlog::{TPMLog, TpmEvent, TpmEventType},
};
use crate::ipc::eve_types::{EfiVariable, TpmLogs};
use anyhow::{anyhow, Result};
use log::{info, trace};

pub struct EveTpmLog {
    pub log: TPMLog,
    pub efi_vars: Option<Vec<EfiVariable>>,
}

impl EveTpmLog {
    pub fn get_events_for_pcr_ref(&self, pcr: u32) -> Vec<&TpmEvent> {
        self.log.events_for_pcr_ref(pcr)
    }
}

pub fn get_logs_pair(raw_logs: TpmLogs) -> Result<(EveTpmLog, EveTpmLog)> {
    let good = raw_logs
        .last_good_log
        .or(raw_logs.backup_good_log)
        .ok_or(anyhow!("'goog' log is missing"))
        .map(|raw_log| -> Result<EveTpmLog> {
            Ok(EveTpmLog {
                log: TPMLog::from_slice(&raw_log)?,
                efi_vars: raw_logs.efi_vars_success,
            })
        })??;

    let bad = raw_logs
        .last_failed_log
        .or(raw_logs.backup_failed_log)
        .ok_or(anyhow!("'bad' log is missing"))
        .map(|raw_log| -> Result<EveTpmLog> {
            Ok(EveTpmLog {
                log: TPMLog::from_slice(&raw_log)?,
                efi_vars: raw_logs.efi_vars_failed,
            })
        })??;

    Ok((good, bad))
}

pub(super) fn tpm_log_compute_lcs<'a>(
    good: &[&'a TpmEvent],
    bad: &[&'a TpmEvent],
) -> Vec<&'a TpmEvent> {
    let good_len = good.len();
    let bad_len = bad.len();

    // Initialize DP table with dimensions (good_len + 1) x (bad_len + 1)
    let mut dp = vec![vec![0; bad_len + 1]; good_len + 1];

    // Fill the DP table
    for i in 1..=good_len {
        for j in 1..=bad_len {
            if good[i - 1] == bad[j - 1] {
                // Events match: extend the LCS
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                // Take the maximum of the left or top cell
                dp[i][j] = std::cmp::max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    // Backtrack to reconstruct the LCS
    let mut i = good_len;
    let mut j = bad_len;
    let mut lcs = Vec::new();

    while i > 0 && j > 0 {
        if good[i - 1] == bad[j - 1] {
            // Include the matching event in the LCS
            lcs.push(good[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            // Move up (prioritize the good log)
            i -= 1;
        } else {
            // Move left (prioritize the bad log)
            j -= 1;
        }
    }

    // Reverse to restore original order
    lcs.reverse();
    lcs
}

pub(super) fn tpm_log_diff_binary<'a>(
    good: &[&'a TpmEvent],
    bad: &[&'a TpmEvent],
    lcs: &[&'a TpmEvent],
) -> (Vec<&'a TpmEvent>, Vec<&'a TpmEvent>) {
    // Find deletions (events in `good` but not in LCS)
    let deletions: Vec<_> = good.iter().filter(|e| !lcs.contains(e)).copied().collect();

    // Find insertions (events in `bad` but not in LCS)
    let insertions: Vec<_> = bad.iter().filter(|e| !lcs.contains(e)).copied().collect();

    (deletions, insertions)
}

pub(super) fn get_event_key(event: &TpmEvent) -> Option<String> {
    match event.event_type {
        TpmEventType::EfiVariableBoot | TpmEventType::EfiVariableBoot2 => {
            let efi_var = EFIVariableBootEvent::parse(&event.event_data).ok()?;
            Some(format!("EFIVar:{}", efi_var.unicode_name))
        }
        TpmEventType::Action => {
            // Use event data as key
            Some(String::from_utf8(event.event_data.clone()).ok()?)
        }
        TpmEventType::IPL if event.pcr_index == 8 => {
            // decode grub event
            let grub_event = GrubEvent::try_from(event).ok()?;
            match grub_event {
                GrubEvent::Cmd(d) => {
                    // split the command into command and the rest
                    let d = d.splitn(2, ' ').next().unwrap_or(&d);
                    Some(format!("GrubCmd:{}", d))
                }
                GrubEvent::KernelCmdLine(_) => Some("GrubKernelCmdLine".to_string()),
                GrubEvent::LinuxEfi(_) => Some("GrubLinuxEfi".to_string()),
            }
        }
        _ => Some(format!("{}", event.event_type)),
    }
}

// Detect simanctic Modifications
// if the same event exists in both deltions and insetions then it is a modification
// e.g. BootOrder changed from [1, 2, 3] to [1, 3, 2]. It is marked as deleted in
// good log and inserted in bad log. However this is the same event with different data.
fn tpm_log_diff_semantic<'a>(
    bin_insertions: &'a Vec<&'a TpmEvent>,
    bin_deletions: &'a Vec<&'a TpmEvent>,
) -> (
    Vec<&'a &'a TpmEvent>,
    Vec<&'a &'a TpmEvent>,
    Vec<(&'a &'a TpmEvent, &'a &'a TpmEvent)>,
) {
    let mut mods = Vec::new();
    let mut del_indexes = Vec::new();
    let mut ins_indexes = Vec::new();

    let mut del_map: HashMap<_, _> = bin_deletions
        .iter()
        .enumerate()
        .filter_map(|(index, e)| get_event_key(e).map(|k| (k, (index, e))))
        .collect();

    println!("{:#?}", &del_map.keys());

    for (index, ins) in bin_insertions.iter().enumerate() {
        if let Some(key) = get_event_key(ins) {
            if let Some((del_index, event)) = del_map.remove(&key) {
                mods.push((event, ins));
                del_indexes.push(del_index);
                ins_indexes.push(index);
            }
        }
    }

    // cleanup deletions and insertions
    let deletions = bin_deletions
        .iter()
        .enumerate()
        .filter(|(index, _)| !del_indexes.contains(index))
        .map(|(_, e)| e)
        .collect::<Vec<_>>();

    let insertions = bin_insertions
        .iter()
        .enumerate()
        .filter(|(index, _)| !ins_indexes.contains(index))
        .map(|(_, e)| e)
        .collect::<Vec<_>>();

    (deletions, insertions, mods)
}

struct InterpretedTpmEvent {
    event: TpmEvent,
    key: Option<String>,
}

pub fn tpm_log_diff_interpret(pcrs: &[u32], good: EveTpmLog, bad: EveTpmLog) {
    for pcr in pcrs {
        // get events for pcr from both logs
        let good_events = good.get_events_for_pcr_ref(*pcr);
        let bad_events = bad.get_events_for_pcr_ref(*pcr);

        let lcs = tpm_log_compute_lcs(&good_events, &bad_events);
        let (deletions, insertions) = tpm_log_diff_binary(&good_events, &bad_events, &lcs);
        let (deletions, insertions, mods) = tpm_log_diff_semantic(&insertions, &deletions);

        // print insertions
        for event in &insertions {
            println!("+ {:?}", event.event_type);
        }

        // print deletions
        for event in &deletions {
            println!("- {:?}", event.event_type);
        }

        // print modifications
        for (old_event, new_event) in &mods {
            println!("M {:?} -> {:?}", old_event.event_type, new_event.event_type);
        }
        panic!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moc_tpm_log(path: &str) -> TPMLog {
        let data = std::fs::read(path).unwrap();
        TPMLog::from_slice(&data).unwrap()
    }

    #[test]
    fn test_decode_tpm_logs_message() {
        // load /home/mikem/projects/monitor/eve-monitor-rs/src/tpm/test_data/pcr8-14/2025-03-04-10-52-35/eve_ipc_message-6.json
        // and deserialize to TpmLogs
        let message =
            std::fs::read("/home/mikem/projects/monitor/eve-monitor-rs/src/tpm/test_data/pcr8/log/2025-03-04-12-25-31/eve_ipc_message-6.json")
                .unwrap();

        let mut json_data: serde_json::Value = serde_json::from_slice(&message).unwrap();

        let raw_logs: TpmLogs =
            serde_json::from_value::<TpmLogs>(json_data["message"].take()).unwrap();

        let (good, bad) = get_logs_pair(raw_logs).unwrap();

        tpm_log_diff_interpret(&[8], good, bad);
    }
}
