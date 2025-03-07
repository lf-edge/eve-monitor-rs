use std::collections::HashMap;

use super::{
    tcg_events::{TcgEfiVariableEvent, TcgIPLEvent},
    tcg_tpmlog::{TPMLog, TcgTpmEvent, TcgTpmEventType},
};
use crate::{
    efi::vars::{EfiBootOrder, EfiLoadOption},
    ipc::eve_types::{EfiVariable, TpmLogs},
    tpm::tcg_events::TcgEfiActionEvent,
};
use anyhow::{anyhow, Result};
use log::{info, trace};

pub struct EveTpmLog {
    pub log: TPMLog,
    pub efi_vars: Option<Vec<EfiVariable>>,
}

impl EveTpmLog {
    pub fn from_events(events: Vec<TcgTpmEvent>) -> Self {
        let log = TPMLog::from_events(events);
        Self {
            log,
            efi_vars: None,
        }
    }
    pub fn get_events_for_pcr_ref(&self, pcr: u32) -> Vec<&TcgTpmEvent> {
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
    good: &[&'a TcgTpmEvent],
    bad: &[&'a TcgTpmEvent],
) -> Vec<&'a TcgTpmEvent> {
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
    good: &[&'a TcgTpmEvent],
    bad: &[&'a TcgTpmEvent],
    lcs: &[&'a TcgTpmEvent],
) -> (Vec<&'a TcgTpmEvent>, Vec<&'a TcgTpmEvent>) {
    // Find deletions (events in `good` but not in LCS)
    let deletions: Vec<_> = good.iter().filter(|e| !lcs.contains(e)).copied().collect();

    // Find insertions (events in `bad` but not in LCS)
    let insertions: Vec<_> = bad.iter().filter(|e| !lcs.contains(e)).copied().collect();

    (deletions, insertions)
}

// pub(super) fn get_event_semantic_key(event: &TcgTpmEvent) -> Option<String> {
//     match event.event_type {
//         TcgTpmEventType::EfiVariableBoot | TcgTpmEventType::EfiVariableBoot2 => {
//             let efi_var = TcgEfiVariableEvent::try_from(event).ok()?;
//             Some(format!("EFIVar:{}", efi_var.unicode_name))
//         }
//         TcgTpmEventType::EfiAction => {
//             // for PCR14 use file name as a key
//             let action = TcgEfiActionEvent::try_from(event).ok()?;
//             match action {
//                 TcgEfiActionEvent::MeasureConfig {
//                     file,
//                     hash: _,
//                     exists: _,
//                 } => Some(format!("Config:{}", file)),
//                 TcgEfiActionEvent::EnterBiosSetup => Some("BIOS".to_string()),
//                 TcgEfiActionEvent::EfiAction(s) => Some(s),
//             }
//         }
//         TcgTpmEventType::IPL if event.pcr_index == 8 => {
//             // decode grub event
//             let grub_event = TcgIPLEvent::try_from(event).ok()?;
//             match grub_event {
//                 TcgIPLEvent::Cmd(d) => {
//                     // split the command into command and the rest
//                     let d = d.splitn(2, ' ').next().unwrap_or(&d);
//                     Some(format!("GrubCmd:{}", d))
//                 }
//                 TcgIPLEvent::KernelCmdLine(_) => Some("GrubKernelCmdLine".to_string()),
//                 TcgIPLEvent::LinuxEfi(_) => Some("GrubLinuxEfi".to_string()),
//             }
//         }
//         _ => Some(format!("{}", event.event_type)),
//     }
// }

// Detect simanctic Modifications
// if the same event exists in both deltions and insetions then it is a modification
// e.g. BootOrder changed from [1, 2, 3] to [1, 3, 2]. It is marked as deleted in
// good log and inserted in bad log. However this is the same event with different data.
// pub(super) fn tpm_log_diff_semantic<'a>(
//     bin_insertions: Vec<&'a TcgTpmEvent>,
//     bin_deletions: Vec<&'a TcgTpmEvent>,
// ) -> (
//     Vec<&'a TcgTpmEvent>,
//     Vec<&'a TcgTpmEvent>,
//     Vec<(&'a TcgTpmEvent, &'a TcgTpmEvent)>,
// ) {
//     let mut mods = Vec::new();
//     let mut del_indexes = Vec::new();
//     let mut ins_indexes = Vec::new();

//     let mut del_map: HashMap<_, _> = bin_deletions
//         .iter()
//         .enumerate()
//         .filter_map(|(index, e)| get_event_semantic_key(e).map(|k| (k, (index, e))))
//         .collect();

//     // println!("{:#?}", &del_map.keys());

//     for (index, ins) in bin_insertions.iter().enumerate() {
//         if let Some(key) = get_event_semantic_key(ins) {
//             if let Some((del_index, event)) = del_map.get(&key) {
//                 mods.push((**event, *ins));
//                 del_indexes.push(*del_index);
//                 ins_indexes.push(index);
//             }
//         }
//     }

//     // cleanup deletions and insertions
//     let deletions = bin_deletions
//         .iter()
//         .enumerate()
//         .filter(|(index, _)| !del_indexes.contains(index))
//         .map(|(_, e)| *e)
//         .collect::<Vec<_>>();

//     let insertions = bin_insertions
//         .iter()
//         .enumerate()
//         .filter(|(index, _)| !ins_indexes.contains(index))
//         .map(|(_, e)| *e)
//         .collect::<Vec<_>>();

//     (deletions, insertions, mods)
// }

#[derive(Debug, PartialEq)]
pub enum ConfigFileStatus {
    Added,
    Deleted,
    Modified,
}

#[derive(Debug)]
pub enum InterpretedTpmEvent {
    ConfigFileModified {
        file: String,
        status: ConfigFileStatus,
    },
    KernelCmdLineModified {
        old: String,
        new: String,
    },
    GrubCfgModified,
    BootOrderModified {
        old: Vec<u32>,
        new: Vec<u32>,
    },
    BootOptionsModified {
        old: Vec<String>,
        new: Vec<String>,
    },
    Error(TcgTpmEvent),
}

// pub fn tpm_log_diff_interpret(
//     pcrs: &[u32],
//     good: EveTpmLog,
//     bad: EveTpmLog,
// ) -> Vec<InterpretedTpmEvent> {
//     let mut pcr_map = pcrs
//         .iter()
//         .map(|pcr| {
//             let good_events = good.get_events_for_pcr_ref(*pcr);
//             let bad_events = bad.get_events_for_pcr_ref(*pcr);

//             let lcs = tpm_log_compute_lcs(&good_events, &bad_events);
//             let (deletions, insertions) = tpm_log_diff_binary(&good_events, &bad_events, &lcs);
//             let (deletions, insertions, mods) = tpm_log_diff_semantic(insertions, deletions);
//             (*pcr, (deletions, insertions, mods))
//         })
//         .collect::<HashMap<_, _>>();

//     let mut interpretations = Vec::new();

//     if let Some((deletions, insertions, mods)) = pcr_map.remove(&14) {
//         interpretations.extend(interpret_pcr14(&deletions, &insertions, &mods));
//     }

//     // let grub_cfg_changed = interpretations.iter().find(|e| match e {
//     //     InterpretedTpmEvent::ConfigFileModified { file, status: _ } => file == "/config/grub.cfg",
//     //     _ => false,
//     // });

//     if let Some((deletions, insertions, events)) = pcr_map.remove(&8) {
//         interpretations.extend(interpret_pcr8(&deletions, &insertions, &events));
//     }

//     if let Some((deletions, insertions, events)) = pcr_map.remove(&1) {
//         interpretations.extend(interpret_pcr1(
//             &deletions,
//             &insertions,
//             &events,
//             good.efi_vars.as_ref(),
//             bad.efi_vars.as_ref(),
//         ));
//     }

//     //TODO: if some PCRs left - interpret them as errors

//     interpretations
// }

// a pair of events represents a single file.
// 1. file may be deleted (exists true->false)
// 2. file may be added (exists false->true)
// 3. file may be modified (exists true->true) and hash is different
// if we cannot decode the event we record the original event. in theory it must not happen
// because we interpret events that were already decoded in get_event_key
// detions and insertions are impossible. Only files measure-config cares about are recoded in PCR14
// if an arbitrary file appears on /config partition it is not recorded in PCR14
// pub(super) fn interpret_pcr14(
//     _deletions: &Vec<&TcgTpmEvent>,
//     _insertions: &Vec<&TcgTpmEvent>,
//     mods: &Vec<(&TcgTpmEvent, &TcgTpmEvent)>,
// ) -> Vec<InterpretedTpmEvent> {
//     let mut results = Vec::new();

//     for (e1, e2) in mods {
//         let action1 = TcgEfiActionEvent::try_from(*e1);
//         let action2 = TcgEfiActionEvent::try_from(*e2);
//         match (action1, action2) {
//             (Ok(a1), Ok(a2)) => match (a1, a2) {
//                 (
//                     TcgEfiActionEvent::MeasureConfig {
//                         file: file1,
//                         hash: hash1,
//                         exists: exists1,
//                     },
//                     TcgEfiActionEvent::MeasureConfig {
//                         file: file2,
//                         hash: hash2,
//                         exists: exists2,
//                     },
//                 ) => {
//                     if file1 != file2 {
//                         results.push(InterpretedTpmEvent::Error((*e1).clone()));
//                         results.push(InterpretedTpmEvent::Error((*e2).clone()));
//                         continue;
//                     }

//                     if exists1 && !exists2 {
//                         results.push(InterpretedTpmEvent::ConfigFileModified {
//                             file: file1,
//                             status: ConfigFileStatus::Deleted,
//                         });
//                     } else if !exists1 && exists2 {
//                         results.push(InterpretedTpmEvent::ConfigFileModified {
//                             file: file1,
//                             status: ConfigFileStatus::Added,
//                         });
//                     } else if exists1 && exists2 && hash1 != hash2 {
//                         results.push(InterpretedTpmEvent::ConfigFileModified {
//                             file: file1,
//                             status: ConfigFileStatus::Modified,
//                         });
//                     }
//                 }
//                 _ => {
//                     results.push(InterpretedTpmEvent::Error((*e1).clone()));
//                     results.push(InterpretedTpmEvent::Error((*e2).clone()));
//                 }
//             },

//             (_, _) => {
//                 results.push(InterpretedTpmEvent::Error((*e1).clone()));
//                 results.push(InterpretedTpmEvent::Error((*e2).clone()));
//             }
//         }
//     }

//     results
// }

// fn interpret_pcr1(
//     deletions: &Vec<&TcgTpmEvent>,
//     insertions: &Vec<&TcgTpmEvent>,
//     mods: &Vec<(&TcgTpmEvent, &TcgTpmEvent)>,
//     good_efi_vars: Option<&Vec<EfiVariable>>,
//     bad_efi_vars: Option<&Vec<EfiVariable>>,
// ) -> Vec<InterpretedTpmEvent> {
//     println!("Interpreting [PCR 1]");

//     let result = Vec::new();

//     // parse inserted events
//     // Bootorder cannot be here
//     for &e in insertions {
//         if e.event_type.is_efi_boot_variable() {
//             // we can unwrap since this is how we got to this point. these events are parsable
//             let efi_var_name = TcgEfiVariableEvent::try_from(e).unwrap().unicode_name;

//             // find EFI variable with the same name
//             let efi_var = bad_efi_vars
//                 .unwrap()
//                 .iter()
//                 .find(|v| v.name == efi_var_name)
//                 .unwrap();

//             if efi_var_name.starts_with("Boot") {
//                 let boot_var = EfiLoadOption::parse(&efi_var.value).unwrap().description;
//             }
//         } else {
//             println!("I {:?}", e.event_type);
//         }
//     }

//     // e1 and e2 describe the same variable by design
//     for (e1, e2) in mods {
//         if e1.event_type == TcgTpmEventType::EfiVariableBoot
//             || e1.event_type == TcgTpmEventType::EfiVariableBoot2
//         {
//             // we can unwrap since this is how we got to this point. these events are parsable
//             let efi_var_name = TcgEfiVariableEvent::try_from(*e1).unwrap().unicode_name;
//         } else {
//             println!("M {:?} -> {:?}", e1.event_type, e2.event_type);
//         }
//     }
//     result
// }

// new events may appear only if
// 1. grub.cfg updated due to EVE update
//  - we can detect this fact by looking at eve version
// 2. user typed commands in grub shell. in this case 'shell:' prefix will be appended to TPM event data
// 3. grub.cfg was modified on /config partition. this can be detected through PCR14
//
// events may disappear only if
// 1. grub.cfg updated due to EVE update
// 2. grub.cfg was modified on /config partition
//
// events are modified if
// 1. user select menu item in grub or manually edit command line
// 2. grub.cfg was modified on /config partition
//
// there is no way to tell from TPM log without parsing grub.cfg what exactly caused changes in kernel command line
// but parsing grub.cfg is too complex
//
// when eve is updated this evet is updated
// - EV_IPL grub_cmd setparams Boot 0.0.0-rucoder_monitor-tpm-log-15ec5037-dirty-2025-03-04.10.17-kvm-amd64
// fn interpret_pcr8(
//     deletions: &Vec<&TcgTpmEvent>,
//     insertions: &Vec<&TcgTpmEvent>,
//     mods: &Vec<(&TcgTpmEvent, &TcgTpmEvent)>,
// ) -> Vec<InterpretedTpmEvent> {
//     let mut results = Vec::new();

//     println!("Interpreting [PCR 8]");

//     let mut grub_cfg_changed = false;

//     for (e1, e2) in mods {
//         let grub_event1 = TcgIPLEvent::try_from(*e1);
//         let grub_event2 = TcgIPLEvent::try_from(*e2);
//         match (grub_event1, grub_event2) {
//             (Ok(g1), Ok(g2)) => match (g1, g2) {
//                 (TcgIPLEvent::Cmd(d1), TcgIPLEvent::Cmd(d2)) => {
//                     println!("M {:?} -> {:?}", d1, d2);
//                     grub_cfg_changed = true;
//                 }
//                 (TcgIPLEvent::KernelCmdLine(d1), TcgIPLEvent::KernelCmdLine(d2)) => {
//                     println!("M {:?} -> {:?}", d1, d2);
//                     results.push(InterpretedTpmEvent::KernelCmdLineModified { old: d1, new: d2 });
//                 }
//                 (TcgIPLEvent::LinuxEfi(d1), TcgIPLEvent::LinuxEfi(d2)) => {
//                     println!("M {:?} -> {:?}", d1, d2);
//                     grub_cfg_changed = true;
//                 }
//                 _ => {
//                     println!("M {:?} -> {:?}", e1.event_type, e2.event_type);
//                     grub_cfg_changed = true;
//                 }
//             },
//             (_, _) => {
//                 println!("M {:?} -> {:?}", e1.event_type, e2.event_type);
//                 results.push(InterpretedTpmEvent::Error((*e1).clone()));
//                 results.push(InterpretedTpmEvent::Error((*e2).clone()));
//             }
//         }
//     }

//     if grub_cfg_changed {
//         results.push(InterpretedTpmEvent::GrubCfgModified);
//     }

//     results
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn moc_tpm_log(path: &str) -> TPMLog {
        let data = std::fs::read(path).unwrap();
        TPMLog::from_slice(&data).unwrap()
    }

    // #[test]
    // fn test_decode_tpm_logs_message() {
    //     // load src/tpm/test_data/pcr8-14/2025-03-04-10-52-35/eve_ipc_message-6.json
    //     // and deserialize to TpmLogs
    //     let message =
    //         std::fs::read("src/tpm/test_data/pcr8/log/2025-03-04-12-25-31/eve_ipc_message-6.json")
    //             .unwrap();

    //     let mut json_data: serde_json::Value = serde_json::from_slice(&message).unwrap();

    //     let raw_logs: TpmLogs =
    //         serde_json::from_value::<TpmLogs>(json_data["message"].take()).unwrap();

    //     let (good, bad) = get_logs_pair(raw_logs).unwrap();

    //     tpm_log_diff_interpret(&[8], good, bad);
    // }
}
