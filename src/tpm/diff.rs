use std::{borrow::Borrow, collections::HashMap};

use super::{
    tcg_tpmlog::{TcgRawTpmEvent, TcgTpmEventRef, TcgTpmLog},
    tpmlog::{EveTpmLog, TpmEvent, TpmEventRef},
};
use crate::{
    efi::{
        device_path::{media::MediaNode, PathNode},
        vars::EfiLoadOption,
    },
    ipc::eve_types::{EfiVariable, TpmLogs},
    lcs::{collect_diff, compute_lcs, produce_diff_ops3_no_reorder, DiffOp},
};
use anyhow::{anyhow, Context, Result};
use log::{info, trace};
use strum::Display;
use uuid::uuid;

pub trait LcsSemanticKey<'a, S>
where
    S: Eq,
{
    fn semantic_key(&'a self) -> S;
}

#[derive(Debug)]
pub struct TpmLogDiff {
    tpm_log_parse_result: Option<Vec<InterpretedTpmEventRef>>,
    old_good_tpm_log: EveTpmLog,
    new_tpm_log: EveTpmLog,
    affected_pcrs: Vec<u32>,
    pub parsed_old: Option<Vec<TpmEventRef>>,
    pub parsed_new: Option<Vec<TpmEventRef>>,
    pub diff_ops_old: Option<Vec<DiffOp>>,
    pub diff_ops_new: Option<Vec<DiffOp>>,
}

impl TpmLogDiff {
    pub fn set_affected_pcrs(&mut self, pcrs: &Vec<u32>) {
        self.affected_pcrs = pcrs.clone();
    }
    fn get_logs_pair(raw_logs: TpmLogs) -> Result<(EveTpmLog, EveTpmLog)> {
        if raw_logs.efi_vars_success.is_none() || raw_logs.efi_vars_failed.is_none() {
            return Err(anyhow!("EFI vars are missing in TPM logs from EVE"));
        }
        let good = raw_logs
            .last_good_log
            .or(raw_logs.backup_good_log)
            .ok_or(anyhow!("'goog' log is missing"))
            .map(|raw_log| -> Result<EveTpmLog> {
                Ok(EveTpmLog::new(
                    TcgTpmLog::from_slice(&raw_log)?,
                    raw_logs.efi_vars_success.unwrap(),
                ))
            })??;

        let bad = raw_logs
            .last_failed_log
            .or(raw_logs.backup_failed_log)
            .ok_or(anyhow!("'bad' log is missing"))
            .map(|raw_log| -> Result<EveTpmLog> {
                Ok(EveTpmLog::new(
                    TcgTpmLog::from_slice(&raw_log)?,
                    raw_logs.efi_vars_failed.unwrap(),
                ))
            })??;

        Ok((good, bad))
    }
    pub fn translate_logs(&mut self) {
        self.parsed_old = Some(self.old_good_tpm_log.tcg_tpm_log_translate().unwrap());
        self.parsed_new = Some(self.new_tpm_log.tcg_tpm_log_translate().unwrap());
    }

    pub fn diff(&mut self) {
        let lcs = compute_lcs(
            self.parsed_old.as_ref().unwrap(),
            self.parsed_new.as_ref().unwrap(),
        );
        let (del, ins) = collect_diff(
            self.parsed_old.as_ref().unwrap(),
            self.parsed_new.as_ref().unwrap(),
            &lcs,
        );
        let (del, new, mods) = diff_semantic_simple(
            self.parsed_old.as_ref().unwrap(),
            self.parsed_new.as_ref().unwrap(),
            &del,
            &ins,
        );
        let (diff_ops_old, diff_ops_new) = produce_diff_ops3_no_reorder(&lcs, &new, &del, &mods);
        self.diff_ops_old = Some(diff_ops_old);
        self.diff_ops_new = Some(diff_ops_new);
    }
}

impl TryFrom<TpmLogs> for TpmLogDiff {
    type Error = anyhow::Error;

    fn try_from(value: TpmLogs) -> std::result::Result<Self, Self::Error> {
        let (old_good_tpm_log, new_tpm_log) = Self::get_logs_pair(value)?;

        Ok(TpmLogDiff {
            tpm_log_parse_result: None,
            old_good_tpm_log,
            new_tpm_log,
            affected_pcrs: Vec::new(),
            parsed_old: None,
            parsed_new: None,
            diff_ops_old: None,
            diff_ops_new: None,
        })
    }
}

// Detect simanctic Modifications
// if the same event exists in both deltions and insetions then it is a modification
// e.g. BootOrder changed from [1, 2, 3] to [1, 3, 2]. It is marked as deleted in
// good log and inserted in bad log. However this is the same event with different data.
pub(super) fn tpm_log_diff_semantic<'a>(
    added_events: Vec<TpmEventRef>,
    deleted_events: Vec<TpmEventRef>,
) -> (
    Vec<TpmEventRef>,
    Vec<TpmEventRef>,
    Vec<(TpmEventRef, TpmEventRef)>,
) {
    let mut mods = Vec::new();
    let mut new_events = Vec::new();

    let mut del_map: HashMap<_, _> = deleted_events
        .into_iter()
        .map(|e| {
            let key = e.event.semantic_key();
            (key, e)
        })
        .collect();

    for new_event in added_events.into_iter() {
        if let Some(old_event) = del_map.remove(&new_event.event.semantic_key()) {
            // LCS is not good when events were reordered
            // only add to mods if events are different
            if old_event.event != new_event.event {
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

pub fn diff_semantic_simple<'a, T, S>(
    old: &'a [T],
    new: &'a [T],
    deleted_events: &[usize],
    added_events: &[usize],
) -> (Vec<usize>, Vec<usize>, Vec<(usize, usize)>)
where
    T: LcsSemanticKey<'a, S> + PartialEq + std::fmt::Display,
    S: Eq + std::hash::Hash + std::fmt::Display,
{
    let mut mods = Vec::new();
    let mut new_events = Vec::new();

    let mut del_map = deleted_events
        .iter()
        .map(|e| {
            let key = old[*e].semantic_key();
            (key, *e)
        })
        .collect::<HashMap<S, usize>>();

    for new_event in added_events.iter() {
        trace!("key: {}", new[*new_event].semantic_key());
        if let Some(old_event) = del_map.remove(&new[*new_event].semantic_key()) {
            // trace!("old[old_event]={}", old[old_event]);
            // trace!("new[*new_event]={}", new[*new_event]);
            // LCS is not good when events were reordered
            // only add to mods if events are different
            if old[old_event] != new[*new_event] {
                mods.push((old_event, *new_event));
            }
        } else {
            new_events.push(*new_event);
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

#[derive(Debug, Clone)]
pub struct InterpretedBootEntry {
    pub boot_num: u16,
    pub description: String,
    pub from_usb: bool,
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
        old: Vec<InterpretedBootEntry>,
        new: Vec<InterpretedBootEntry>,
    },
    EnterBios,
    Error,
}

#[derive(Debug, Clone)]
pub struct InterpretedTpmEventRef {
    pub pcr: u32,
    pub old_original_index: usize,
    pub new_original_index: usize,
    pub event: InterpretedTpmEvent,
}

// pub fn tpm_log_diff_interpret(
//     pcrs: &[u32],
//     old_good: &EveTpmLog,
//     new: &EveTpmLog,
// ) -> Result<Vec<InterpretedTpmEventRef>> {
//     info!("tpm_log_diff_interpret");
//     let mut interpretations: Vec<InterpretedTpmEventRef> = Vec::new();

//     for pcr in pcrs {
//         let (deleted, added, mods) = tpm_log_diff_for_pcr(&old_good, &new, *pcr)?;
//         info!("====  PCR: {:?} ====", pcr);
//         // print deleted
//         for e in &deleted {
//             info!("D {:?}", e);
//         }
//         // print added
//         for e in &added {
//             match &e.event {
//                 TpmEvent::BootApplication(dp) => {
//                     info!("BootApplication dp {}", dp.display(false));
//                 }
//                 _ => {}
//             }
//             info!("I {:?}", e);
//         }
//         // print mods
//         for (e1, e2) in &mods {
//             info!("M {:?} -> {:?}", e1, e2);
//         }

//         match pcr {
//             13 => {
//                 interpretations.extend(interpret_pcr14(deleted, added, mods));
//             }
//             8 => {
//                 interpretations.extend(interpret_pcr8(deleted, added, mods));
//             }
//             1 => {
//                 // interpretations.extend(interpret_pcr1(
//                 //     deleted,
//                 //     added,
//                 //     mods,
//                 //     &old_good.efi_vars,
//                 //     &new.efi_vars,
//                 // ));
//             }
//             14 => {
//                 interpretations.extend(interpret_pcr14(deleted, added, mods));
//             }
//             4 => {
//                 interpretations.extend(interpret_pcr4(deleted, added, mods));
//             }
//             _ => {
//                 // let errors = deleted
//                 //     .into_iter()
//                 //     .chain(added)
//                 //     .map(|e| InterpretedTpmEvent::Error(e.event))
//                 //     .collect::<Vec<InterpretedTpmEvent>>();
//                 // if !errors.is_empty() {
//                 //     interpretations.insert(*pcr, errors);
//                 // }
//             }
//         }
//     }

//     Ok(interpretations)
// }

// pub(super) fn tpm_log_diff_for_pcr<'a, 'b>(
//     old_good: &'a EveTpmLog,
//     new: &'b EveTpmLog,
//     pcr: u32,
// ) -> Result<
//     (
//         Vec<TpmEventRef>,
//         Vec<TpmEventRef>,
//         Vec<(TpmEventRef, TpmEventRef)>,
//     ),
//     anyhow::Error,
// > {
//     let good_events = old_good.get_events_for_pcr_ref(pcr);
//     let bad_events = new.get_events_for_pcr_ref(pcr);
//     let lcs = compute_lcs(&good_events, &bad_events);

//     // print LCS
//     info!("==== LCS ====");
//     for e in &lcs {
//         trace!("{:?}", e);
//     }

//     let (deleted_events, added_events) = collect_diff(&good_events, &bad_events, &lcs);

//     // print deleted
//     info!("==== DELETED ====");
//     for e in &deleted_events {
//         trace!("{:?}", e);
//     }

//     // print added
//     info!("==== ADDED ====");
//     for e in &added_events {
//         trace!("{:?}", e);
//     }

//     let deleted_events = tcg_tpm_log_translate(deleted_events, old_good.efi_vars.as_ref())
//         .context("cannot translate deleted events")?;
//     let added_events = tcg_tpm_log_translate(added_events, new.efi_vars.as_ref())
//         .context("cannot translate added events")?;
//     let (deleted, added, mods) = tpm_log_diff_semantic(added_events, deleted_events);
//     Ok((deleted, added, mods))
// }

impl Default for InterpretedTpmEventRef {
    fn default() -> Self {
        Self {
            pcr: 255,
            old_original_index: 0,
            new_original_index: 0,
            event: InterpretedTpmEvent::Error,
        }
    }
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
    _deleted_events: Vec<TpmEventRef>,
    _added_events: Vec<TpmEventRef>,
    mods: Vec<(TpmEventRef, TpmEventRef)>,
) -> Vec<InterpretedTpmEventRef> {
    let mut results = Vec::new();

    for (e1, e2) in mods.into_iter() {
        let mut event_ref = InterpretedTpmEventRef::default();

        event_ref.pcr = 14;
        event_ref.old_original_index = e1.original_index;
        event_ref.new_original_index = e2.original_index;
        match (e1.event, e2.event) {
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
                    event_ref.event = InterpretedTpmEvent::Error;
                } else if exists1 && !exists2 {
                    event_ref.event = InterpretedTpmEvent::ConfigFileModified {
                        file: file1,
                        status: ConfigFileStatus::Deleted,
                    };
                } else if !exists1 && exists2 {
                    event_ref.event = InterpretedTpmEvent::ConfigFileModified {
                        file: file1,
                        status: ConfigFileStatus::Added,
                    };
                } else if exists1 && exists2 && hash1 != hash2 {
                    event_ref.event = InterpretedTpmEvent::ConfigFileModified {
                        file: file1,
                        status: ConfigFileStatus::Modified,
                    };
                }
            }
            (a, b) => {
                event_ref.event = InterpretedTpmEvent::Error;
            }
        }
        results.push(event_ref);
    }

    results
}

fn is_usb_device_path(dp: &Vec<PathNode>) -> bool {
    true
}

fn parse_boot_variable(var: &EfiVariable) -> Result<InterpretedBootEntry> {
    let efi_load_options = EfiLoadOption::try_from(var.value.as_slice())
        .context(format!("cannot parse {}", var.name))?;

    Ok(InterpretedBootEntry {
        boot_num: u16::from_str_radix(&var.name[4..], 16)?,
        description: efi_load_options.description,
        from_usb: is_usb_device_path(&efi_load_options.device_path_list.nodes),
    })
}

fn interpret_pcr1(
    deleted_events: Vec<TpmEventRef>,
    new_events: Vec<TpmEventRef>,
    mods: Vec<(TpmEventRef, TpmEventRef)>,
    old_efi_vars: &Option<Vec<EfiVariable>>,
    new_efi_vars: &Option<Vec<EfiVariable>>,
) -> Vec<InterpretedTpmEventRef> {
    let mut boot_options_changed = false;

    let mut result = Vec::new();

    // collect old and new boot entries
    let old_boot_entries = old_efi_vars.as_ref().map(|v| {
        v.iter()
            .filter_map(|var| {
                if var.name != "BootOrder" {
                    parse_boot_variable(var).ok()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    });

    let new_boot_entries = new_efi_vars.as_ref().map(|v| {
        v.iter()
            .filter_map(|var| {
                if var.name != "BootOrder" {
                    parse_boot_variable(var).ok()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    });

    for e in deleted_events {
        match e.event {
            TpmEvent::BootEntry {
                boot_num: _,
                description: _,
                device_path: _,
                attributes: _,
            } => {
                boot_options_changed = true;
            }
            _ => {
                let mut event_ref = InterpretedTpmEventRef::default();
                event_ref.pcr = 1;
                event_ref.old_original_index = e.original_index;
                result.push(event_ref);
            }
        }
    }

    for e in new_events {
        match e.event {
            TpmEvent::BootEntry {
                boot_num: _,
                description: _,
                device_path: _,
                attributes: _,
            } => {
                boot_options_changed = true;
            }
            _ => {
                let mut event_ref = InterpretedTpmEventRef::default();
                event_ref.new_original_index = e.original_index;
                event_ref.pcr = 1;
                result.push(event_ref);
            }
        }
    }

    let mut old_boot_option_indexes = Vec::new();
    let mut new_boot_option_indexes = Vec::new();

    // modified events
    for (e1, e2) in mods.iter() {
        let mut event_ref = InterpretedTpmEventRef::default();
        event_ref.pcr = 1;
        event_ref.old_original_index = e1.original_index;
        event_ref.new_original_index = e2.original_index;
        match (&e1.event, &e2.event) {
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
                boot_options_changed = true;
                old_boot_option_indexes.push(e1.original_index);
                new_boot_option_indexes.push(e2.original_index);
            }
            (TpmEvent::BootOrder(o1), TpmEvent::BootOrder(o2)) => {
                event_ref.event = InterpretedTpmEvent::BootOrderModified {
                    old: o1.clone(),
                    new: o2.clone(),
                };
            }
            _ => {
                event_ref.event = InterpretedTpmEvent::Error;
            }
        }
        result.push(event_ref);
    }

    if boot_options_changed {
        let old_boot_entries = old_boot_entries.unwrap_or_default();
        let new_boot_entries = new_boot_entries.unwrap_or_default();

        let min_old_index = old_boot_option_indexes.iter().min().unwrap_or(&0);
        let min_new_index = new_boot_option_indexes.iter().min().unwrap_or(&0);

        result.push(InterpretedTpmEventRef {
            pcr: 1,
            old_original_index: *min_old_index,
            new_original_index: *min_new_index,
            event: InterpretedTpmEvent::BootOptionsModified {
                old: old_boot_entries,
                new: new_boot_entries,
            },
        });
    }

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
    _deletions: Vec<TpmEventRef>,
    _insertions: Vec<TpmEventRef>,
    mods: Vec<(TpmEventRef, TpmEventRef)>,
) -> Vec<InterpretedTpmEventRef> {
    let mut results = Vec::new();

    let mut grub_cfg_changed = false;

    for (e1, e2) in mods {
        let mut event_ref = InterpretedTpmEventRef::default();
        event_ref.pcr = 8;
        event_ref.old_original_index = e1.original_index;
        event_ref.new_original_index = e2.original_index;
        match (e1.event, e2.event) {
            (TpmEvent::GrubKernelCmdline(d1), TpmEvent::GrubKernelCmdline(d2)) => {
                event_ref.event = InterpretedTpmEvent::KernelCmdLineModified { old: d1, new: d2 };
                results.push(event_ref);
            }
            (TpmEvent::GrubCmd { cmd: _, params: _ }, TpmEvent::GrubCmd { cmd: _, params: _ }) => {
                grub_cfg_changed = true;
            }
            (TpmEvent::GrubLinuxEfi(_), TpmEvent::GrubLinuxEfi(_)) => {
                grub_cfg_changed = true;
            }
            (e1, e2) => {
                event_ref.event = InterpretedTpmEvent::Error;
            }
        }
    }

    if grub_cfg_changed {
        let mut event_ref = InterpretedTpmEventRef::default();
        event_ref.event = InterpretedTpmEvent::GrubCfgModified;
        results.push(event_ref);
    }

    results
}

fn interpret_pcr4(
    _deletions: Vec<TpmEventRef>,
    insertions: Vec<TpmEventRef>,
    _mods: Vec<(TpmEventRef, TpmEventRef)>,
) -> Vec<InterpretedTpmEventRef> {
    let mut reult = Vec::new();
    for e in insertions {
        let mut event_ref = InterpretedTpmEventRef::default();
        event_ref.new_original_index = e.original_index;
        event_ref.pcr = 4;

        match e.event {
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
                    event_ref.event = InterpretedTpmEvent::EnterBios;
                } else {
                    event_ref.event = InterpretedTpmEvent::Error;
                }
            }
            _ => {
                info!("I {:?}", e);
                event_ref.event = InterpretedTpmEvent::Error;
            }
        }
        reult.push(event_ref);
    }

    reult
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn moc_tpm_log(path: &str) -> TcgTpmLog {
//         let data = std::fs::read(path).unwrap();
//         TcgTpmLog::from_slice(&data).unwrap()
//     }

//     #[test]
//     fn test_decode_tpm_logs_message() -> Result<()> {
//         // init logger
//         let _ = env_logger::builder()
//             .is_test(true)
//             .filter_level(log::LevelFilter::Trace)
//             .try_init();

//         // load src/tpm/test_data/pcr8-14/2025-03-04-10-52-35/eve_ipc_message-6.json
//         // and deserialize to TpmLogs
//         // let message =
//         //     std::fs::read("src/tpm/test_data/pcr8/log/2025-03-04-12-25-31/eve_ipc_message-6.json")
//         //         .unwrap();

//         let message =
//             std::fs::read("/home/mikem/projects/monitor/eve-monitor-rs/persist/monitor/log/2025-03-13-00-07-35/eve_ipc_message-7.json")
//                 .unwrap();

//         let mut json_data: serde_json::Value = serde_json::from_slice(&message).unwrap();

//         let raw_logs: TpmLogs =
//             serde_json::from_value::<TpmLogs>(json_data["message"].take()).unwrap();

//         raw_logs.save_raw_binary_logs(
//             "/home/mikem/projects/monitor/eve-monitor-rs/persist/monitor/log",
//         )?;

//         let (good, bad) = get_logs_pair(raw_logs).unwrap();

//         let (deleted, added, mods) = tpm_log_diff_for_pcr(&good, &bad, 1).unwrap();

//         // print deleted
//         for e in &deleted {
//             info!("D {:?}", e);
//         }

//         // print added
//         for e in &added {
//             info!("I {:?}", e);
//         }
//         // print mods
//         for (e1, e2) in &mods {
//             info!("M {:?} -> {:?}", e1, e2);
//         }

//         //tpm_log_diff_interpret(&[1, 4, 14], good, bad)?;
//         Ok(())
//     }
// }
