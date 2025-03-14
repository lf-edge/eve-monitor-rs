use std::collections::HashMap;

use super::{
    tcg_tpmlog::{TPMLog, TcgTpmEvent},
    tpmlog::{TpmEvent, TpmEventDescribe},
};
use crate::{
    efi::device_path::{media::MediaNode, PathNode},
    ipc::eve_types::{EfiVariable, TpmLogs},
    lcs::{collect_diff, compute_lcs},
};
use anyhow::{anyhow, Context, Result};
use color_eyre::owo_colors::OwoColorize;
use log::{info, trace};
use strum::Display;
use uuid::uuid;

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

pub(super) fn tcg_tpm_log_translate(
    tcg_log: &[&TcgTpmEvent],
    efi_vars: Option<&Vec<EfiVariable>>,
) -> Result<Vec<TpmEvent>> {
    let mut events = Vec::new();

    for event in tcg_log {
        let tpm_event =
            TpmEvent::try_from_tcg_event(event, efi_vars).context("try_from_tcg_event failed")?;
        events.push(tpm_event);
    }

    Ok(events)
}

// Detect simanctic Modifications
// if the same event exists in both deltions and insetions then it is a modification
// e.g. BootOrder changed from [1, 2, 3] to [1, 3, 2]. It is marked as deleted in
// good log and inserted in bad log. However this is the same event with different data.
pub(super) fn tpm_log_diff_semantic<'a>(
    added_events: Vec<TpmEvent>,
    deleted_events: Vec<TpmEvent>,
) -> (Vec<TpmEvent>, Vec<TpmEvent>, Vec<(TpmEvent, TpmEvent)>) {
    let mut mods = Vec::new();
    let mut new_events = Vec::new();

    let mut del_map: HashMap<_, _> = deleted_events
        .into_iter()
        .map(|e| {
            let key = e.semantic_key();
            (key, e)
        })
        .collect();

    for new_event in added_events.into_iter() {
        if let Some(old_event) = del_map.remove(&new_event.semantic_key()) {
            // LCS is not good when events were reordered
            // only add to mods if events are different
            if old_event != new_event {
                mods.push((old_event, new_event));
            }
        } else {
            new_events.push(new_event);
        }
    }

    // what is left in hashmap are real deleted events
    // FIXME: do we care about the order?
    let deleted_events = del_map.into_values().collect::<Vec<_>>();

    (deleted_events, new_events, mods)
}

#[derive(Debug, PartialEq, Display, Clone)]
pub enum ConfigFileStatus {
    Added,
    Deleted,
    Modified,
}

#[derive(Debug, Display, Clone)]
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
        old: Vec<u16>,
        new: Vec<u16>,
    },
    BootOptionsModified {
        old: Vec<String>,
        new: Vec<String>,
    },
    EnterBios,
    Error(TpmEvent),
}

pub fn tpm_log_diff_interpret(
    pcrs: &[u32],
    old_good: EveTpmLog,
    new: EveTpmLog,
) -> Result<Vec<(u32, Vec<InterpretedTpmEvent>)>> {
    info!("tpm_log_diff_interpret");
    let mut interpretations: HashMap<u32, Vec<InterpretedTpmEvent>> = HashMap::new();

    for pcr in pcrs {
        let (deleted, added, mods) = tpm_log_diff_for_pcr(&old_good, &new, *pcr)?;
        info!("====  PCR: {:?} ====", pcr);
        // print deleted
        for e in &deleted {
            info!("D {:?}", e);
        }
        // print added
        for e in &added {
            match e {
                TpmEvent::BootApplication(dp) => {
                    info!("BootApplication dp {}", dp.display(false));
                }
                _ => {}
            }
            info!("I {:?}", e);
        }
        // print mods
        for (e1, e2) in &mods {
            info!("M {:?} -> {:?}", e1, e2);
        }

        match pcr {
            13 => {
                interpretations.insert(13, interpret_pcr14(deleted, added, mods));
            }
            8 => {
                interpretations.insert(8, interpret_pcr8(deleted, added, mods));
            }
            1 => {
                interpretations.insert(1, interpret_pcr1(deleted, added, mods));
            }
            14 => {
                interpretations.insert(14, interpret_pcr14(deleted, added, mods));
            }
            4 => {
                interpretations.insert(4, interpret_pcr4(deleted, added, mods));
            }
            _ => {
                let errors = deleted
                    .into_iter()
                    .chain(added)
                    .map(|e| InterpretedTpmEvent::Error(e))
                    .collect::<Vec<InterpretedTpmEvent>>();
                if !errors.is_empty() {
                    interpretations.insert(*pcr, errors);
                }
            }
        }
    }

    let mut interpretations = interpretations.into_iter().collect::<Vec<_>>();
    interpretations.sort_by_key(|e| e.0);

    Ok(interpretations)
}

pub(super) fn tpm_log_diff_for_pcr<'a, 'b>(
    old_good: &'a EveTpmLog,
    new: &'b EveTpmLog,
    pcr: u32,
) -> Result<(Vec<TpmEvent>, Vec<TpmEvent>, Vec<(TpmEvent, TpmEvent)>), anyhow::Error> {
    let good_events = old_good.get_events_for_pcr_ref(pcr);
    let bad_events = new.get_events_for_pcr_ref(pcr);
    let lcs = compute_lcs(&good_events, &bad_events);

    // print LCS
    info!("==== LCS ====");
    for (e) in &lcs {
        trace!("{:?}", e);
    }

    let (deleted_events, added_events) = collect_diff(&good_events, &bad_events, &lcs);

    // print deleted
    info!("==== DELETED ====");
    for e in &deleted_events {
        trace!("{:?}", e);
    }

    // print added
    info!("==== ADDED ====");
    for e in &added_events {
        trace!("{:?}", e);
    }

    let deleted_events = tcg_tpm_log_translate(&deleted_events, old_good.efi_vars.as_ref())
        .context("cannot translate deleted events")?;
    let added_events = tcg_tpm_log_translate(&added_events, new.efi_vars.as_ref())
        .context("cannot translate added events")?;
    let (deleted, added, mods) = tpm_log_diff_semantic(added_events, deleted_events);
    Ok((deleted, added, mods))
}

// a pair of events represents a single file.
// 1. file may be deleted (exists true->false)
// 2. file may be added (exists false->true)
// 3. file may be modified (exists true->true) and hash is different
// if we cannot decode the event we record the original event. in theory it must not happen
// because we interpret events that were already decoded in get_event_key
// detions and insertions are impossible. Only files measure-config cares about are recoded in PCR14
// if an arbitrary file appears on /config partition it is not recorded in PCR14
pub(super) fn interpret_pcr14(
    _deleted_events: Vec<TpmEvent>,
    _added_events: Vec<TpmEvent>,
    mods: Vec<(TpmEvent, TpmEvent)>,
) -> Vec<InterpretedTpmEvent> {
    let mut results = Vec::new();

    for (e1, e2) in mods.into_iter() {
        match (e1, e2) {
            (
                TpmEvent::MeasureConfig {
                    file: file1,
                    hash: hash1,
                    exists: exists1,
                },
                TpmEvent::MeasureConfig {
                    file: file2,
                    hash: hash2,
                    exists: exists2,
                },
            ) => {
                if file1 != file2 {
                    results.push(InterpretedTpmEvent::Error(TpmEvent::MeasureConfig {
                        file: file1,
                        hash: hash1,
                        exists: exists1,
                    }));
                    results.push(InterpretedTpmEvent::Error(TpmEvent::MeasureConfig {
                        file: file2,
                        hash: hash2,
                        exists: exists2,
                    }));
                    continue;
                }

                if exists1 && !exists2 {
                    results.push(InterpretedTpmEvent::ConfigFileModified {
                        file: file1,
                        status: ConfigFileStatus::Deleted,
                    });
                } else if !exists1 && exists2 {
                    results.push(InterpretedTpmEvent::ConfigFileModified {
                        file: file1,
                        status: ConfigFileStatus::Added,
                    });
                } else if exists1 && exists2 && hash1 != hash2 {
                    results.push(InterpretedTpmEvent::ConfigFileModified {
                        file: file1,
                        status: ConfigFileStatus::Modified,
                    });
                }
            }
            (a, b) => {
                results.push(InterpretedTpmEvent::Error(a));
                results.push(InterpretedTpmEvent::Error(b));
            }
        }
    }

    results
}

fn interpret_pcr1(
    deleted_events: Vec<TpmEvent>,
    new_events: Vec<TpmEvent>,
    mods: Vec<(TpmEvent, TpmEvent)>,
) -> Vec<InterpretedTpmEvent> {
    //println!("Interpreting [PCR 1]");

    let mut result = Vec::new();

    for e in deleted_events {
        match e {
            TpmEvent::BootEntry {
                boot_num,
                description,
                device_path,
                attributes,
            } => {
                //println!("D {} {} {:?}", boot_num, description, device_path);
            }
            // TpmEvent::BootOrder(items) => {
            //     // println!("D {:?}", items);
            // }
            _ => {
                //println!("D {:?}", e);
            }
        }
    }

    for e in new_events {
        match e {
            TpmEvent::BootEntry {
                boot_num,
                description,
                device_path,
                attributes,
            } => {
                //println!("I {} {} {:?}", boot_num, description, device_path);
            }
            _ => {
                //println!("I {:?}", e);
            }
        }
    }

    // modifed events
    for (e1, e2) in mods {
        match (e1, e2) {
            (
                TpmEvent::BootEntry {
                    boot_num: boot_num1,
                    description: description1,
                    device_path: device_path1,
                    attributes: attributes1,
                },
                TpmEvent::BootEntry {
                    boot_num: boot_num2,
                    description: description2,
                    device_path: device_path2,
                    attributes: attributes2,
                },
            ) => {
                //println!("M {} {} {:?}", boot_num1, description1, device_path1);
                //println!("M {} {} {:?}", boot_num2, description2, device_path2);
            }
            (TpmEvent::BootOrder(o1), TpmEvent::BootOrder(o2)) => {
                result.push(InterpretedTpmEvent::BootOrderModified { old: o1, new: o2 });
            }
            _ => {
                //println!("M {:?} -> {:?}", e1, e2);
            }
        }
    }

    // parse inserted events
    // Bootorder cannot be here
    // for e in insertions {
    //     if e.event_type.is_efi_boot_variable() {
    //         // we can unwrap since this is how we got to this point. these events are parsable
    //         let efi_var_name = TcgEfiVariableEvent::try_from(e).unwrap().unicode_name;

    //         // find EFI variable with the same name
    //         let efi_var = bad_efi_vars
    //             .unwrap()
    //             .iter()
    //             .find(|v| v.name == efi_var_name)
    //             .unwrap();

    //         if efi_var_name.starts_with("Boot") {
    //             let boot_var = EfiLoadOption::parse(&efi_var.value).unwrap().description;
    //         }
    //     } else {
    //         println!("I {:?}", e.event_type);
    //     }
    // }

    // e1 and e2 describe the same variable by design
    // for (e1, e2) in mods {
    //     if e1.event_type == TcgTpmEventType::EfiVariableBoot
    //         || e1.event_type == TcgTpmEventType::EfiVariableBoot2
    //     {
    //         // we can unwrap since this is how we got to this point. these events are parsable
    //         let efi_var_name = TcgEfiVariableEvent::try_from(*e1).unwrap().unicode_name;
    //     } else {
    //         println!("M {:?} -> {:?}", e1.event_type, e2.event_type);
    //     }
    // }
    result
}

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
fn interpret_pcr8(
    _deletions: Vec<TpmEvent>,
    _insertions: Vec<TpmEvent>,
    mods: Vec<(TpmEvent, TpmEvent)>,
) -> Vec<InterpretedTpmEvent> {
    let mut results = Vec::new();

    //println!("Interpreting [PCR 8]");

    let mut grub_cfg_changed = false;

    for (e1, e2) in mods {
        match (e1, e2) {
            (TpmEvent::GrubKernelCmdline(d1), TpmEvent::GrubKernelCmdline(d2)) => {
                results.push(InterpretedTpmEvent::KernelCmdLineModified { old: d1, new: d2 });
            }
            (TpmEvent::GrubCmd { cmd: _, params: _ }, TpmEvent::GrubCmd { cmd: _, params: _ }) => {
                grub_cfg_changed = true;
            }
            (TpmEvent::GrubLinuxEfi(_), TpmEvent::GrubLinuxEfi(_)) => {
                grub_cfg_changed = true;
            }
            (e1, e2) => {
                results.push(InterpretedTpmEvent::Error(e1));
                results.push(InterpretedTpmEvent::Error(e2));
            }
        }
    }

    if grub_cfg_changed {
        results.push(InterpretedTpmEvent::GrubCfgModified);
    }

    results
}

fn interpret_pcr4(
    _deletions: Vec<TpmEvent>,
    insertions: Vec<TpmEvent>,
    _mods: Vec<(TpmEvent, TpmEvent)>,
) -> Vec<InterpretedTpmEvent> {
    let mut reult = Vec::new();
    for e in insertions {
        match e {
            TpmEvent::CallingEfiAppFromBootOption | TpmEvent::FailedToStartEfiAppFromBootOption => {
                // just skip it. there is no easy way to know which app exactly so we cannot
                // reliably distinguish between two identical events
            }
            TpmEvent::BootApplication(ref dp) => {
                info!("BootApplication dp {}", dp.display(false));
                let bios_uuids = vec![
                    uuid!("462CAA21-7614-4503-836E-8AB6F4662331"),
                    uuid!("D89A7D8B-D016-4D26-93E3-EAB6B4D3B0A2"),
                    uuid!("EEC25BDC-67F2-4D95-B1D5-F81B2039D11D"),
                ];
                let is_bios = dp.nodes.iter().any(|e| -> bool {
                    match e {
                        PathNode::Media(MediaNode::FvFile(uuid)) => bios_uuids.contains(uuid),
                        _ => false,
                    }
                });
                if is_bios {
                    reult.push(InterpretedTpmEvent::EnterBios);
                } else {
                    reult.push(InterpretedTpmEvent::Error(e));
                }
            }
            _ => {
                info!("I {:?}", e);
                reult.push(InterpretedTpmEvent::Error(e));
            }
        }
    }

    reult
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moc_tpm_log(path: &str) -> TPMLog {
        let data = std::fs::read(path).unwrap();
        TPMLog::from_slice(&data).unwrap()
    }

    #[test]
    fn test_decode_tpm_logs_message() -> Result<()> {
        // init logger
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();

        // load src/tpm/test_data/pcr8-14/2025-03-04-10-52-35/eve_ipc_message-6.json
        // and deserialize to TpmLogs
        // let message =
        //     std::fs::read("src/tpm/test_data/pcr8/log/2025-03-04-12-25-31/eve_ipc_message-6.json")
        //         .unwrap();

        let message =
            std::fs::read("/home/mikem/projects/monitor/eve-monitor-rs/persist/monitor/log/2025-03-13-00-07-35/eve_ipc_message-7.json")
                .unwrap();

        let mut json_data: serde_json::Value = serde_json::from_slice(&message).unwrap();

        let raw_logs: TpmLogs =
            serde_json::from_value::<TpmLogs>(json_data["message"].take()).unwrap();

        raw_logs.save_raw_binary_logs(
            "/home/mikem/projects/monitor/eve-monitor-rs/persist/monitor/log",
        )?;

        let (good, bad) = get_logs_pair(raw_logs).unwrap();

        let (deleted, added, mods) = tpm_log_diff_for_pcr(&good, &bad, 1).unwrap();

        // print deleted
        for e in &deleted {
            info!("D {:?}", e);
        }

        // print added
        for e in &added {
            info!("I {:?}", e);
        }
        // print mods
        for (e1, e2) in &mods {
            info!("M {:?} -> {:?}", e1, e2);
        }

        //tpm_log_diff_interpret(&[1, 4, 14], good, bad)?;
        Ok(())
    }
}
