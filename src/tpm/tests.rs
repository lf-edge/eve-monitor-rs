// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    lcs::{collect_diff, compute_lcs},
    tpm::{
        diff::{interpret_pcr14, tpm_log_diff_for_pcr, InterpretedTpmEvent},
        tcg_events::TcgEfiVariableEvent,
        tcg_tpmlog::{Digest, TPMLog, TcgTpmEvent, TcgTpmEventType},
        tpmlog::TpmEvent,
    },
};

use super::diff::EveTpmLog;

// Helper to create EFI variable boot events
fn mock_boot_order_event(order: &[u16], is_type_2: bool) -> TcgTpmEvent {
    let efi_var = TcgEfiVariableEvent {
        vendor_guid: Uuid::parse_str("8BE4DF61-93CA-11D2-AA0D-00E098032B8C").unwrap(), // EFI_GLOBAL_VARIABLE_GUID
        unicode_name: "BootOrder".to_string(),
        variable_data: order
            .iter()
            .flat_map(|v| v.to_le_bytes().to_vec())
            .collect(),
    };

    let event_data = efi_var.serialize();

    if is_type_2 {
        return mock_tcg_tpm_event(1, TcgTpmEventType::EfiVariableBoot2, &event_data);
    } else {
        return mock_tcg_tpm_event(1, TcgTpmEventType::EfiVariableBoot, &event_data);
    }
}

fn moc_boot_event(index: u16, is_type_2: bool) -> TcgTpmEvent {
    let efi_var = TcgEfiVariableEvent {
        vendor_guid: Uuid::parse_str("8BE4DF61-93CA-11D2-AA0D-00E098032B8C").unwrap(), // EFI_GLOBAL_VARIABLE_GUID
        unicode_name: format!("Boot{:04X}", index),
        variable_data: index.to_le_bytes().to_vec(),
    };

    let event_data = efi_var.serialize();

    if is_type_2 {
        return mock_tcg_tpm_event(1, TcgTpmEventType::EfiVariableBoot2, &event_data);
    } else {
        return mock_tcg_tpm_event(1, TcgTpmEventType::EfiVariableBoot, &event_data);
    }
}

fn moc_pcr14_event(file: &str, exists: bool, hash: Option<&str>) -> TcgTpmEvent {
    let event_data = if let Some(hash) = hash {
        format!("file:{} exist:{} content-hash:{}", file, exists, hash)
    } else {
        format!("file:{} exist:{}", file, exists)
    };
    return mock_tcg_tpm_event(14, TcgTpmEventType::EfiAction, &event_data);
}

// Helper to create mock events
fn mock_tcg_tpm_event<T>(pcr: u32, event_type: TcgTpmEventType, data: &T) -> TcgTpmEvent
where
    T: AsRef<[u8]> + ?Sized,
{
    TcgTpmEvent {
        pcr_index: pcr,
        event_type,
        digests: vec![Digest::new_sha256(data.as_ref())], // Fixed digests
        event_data: data.as_ref().to_vec(),
    }
}

#[test]
fn test_tpmlog() -> Result<()> {
    let data =
        std::fs::read("./src/tpm/test_data/add-uefi-shell/tpm_measurement_seal_success").unwrap();
    let log = TPMLog::from_slice(&data).unwrap();
    println!();
    for event in log.events {
        if event.pcr_index == 1 {
            println!("PCR: {} {:?}", event.pcr_index, event.event_type);
            if event.event_type == TcgTpmEventType::EfiVariableBoot {
                // convert event data to EFIVariableBootEvent
                let _efi_event = TcgEfiVariableEvent::try_from(&event)?;
            }
        }
    }
    Ok(())
}

#[test]
fn test_lcs_insertion() {
    // --- Create Mock Events ---
    // "Good" log events: A, B, C, D
    let good_events = vec![
        mock_tcg_tpm_event(1, TcgTpmEventType::NoAction, "EventA"),
        mock_tcg_tpm_event(1, TcgTpmEventType::PostCode, "EventB"),
        mock_tcg_tpm_event(1, TcgTpmEventType::Separator, "EventC"),
        mock_tcg_tpm_event(1, TcgTpmEventType::Action, "EventD"),
    ];

    // "Bad" log events: A, B, *E*, C, D (E inserted)
    let bad_events = vec![
        mock_tcg_tpm_event(1, TcgTpmEventType::NoAction, "EventA"),
        mock_tcg_tpm_event(1, TcgTpmEventType::PostCode, "EventB"),
        mock_tcg_tpm_event(1, TcgTpmEventType::EventTag, "EventE"), // Inserted
        mock_tcg_tpm_event(1, TcgTpmEventType::Separator, "EventC"),
        mock_tcg_tpm_event(1, TcgTpmEventType::Action, "EventD"),
    ];

    // --- Compute LCS ---
    let lcs = compute_lcs(&good_events, &bad_events);

    // --- Verify LCS ---
    // Expected LCS: A, B, C, D (excluding inserted E)
    assert_eq!(lcs.len(), 4);
    assert_eq!(lcs[0].event_data, b"EventA");
    assert_eq!(lcs[1].event_data, b"EventB");
    assert_eq!(lcs[2].event_data, b"EventC");
    assert_eq!(lcs[3].event_data, b"EventD");

    // --- Find Differences ---
    let (deletions, insertions) = collect_diff(&good_events, &bad_events, &lcs);

    // --- Assertions ---
    // 1. No deletions expected
    assert!(
        deletions.is_empty(),
        "Unexpected deletions: {:?}",
        deletions
    );

    // 2. One insertion expected (EventE)
    assert_eq!(insertions.len(), 1, "Expected 1 insertion");
    let inserted = insertions[0];
    assert_eq!(inserted.event_type, TcgTpmEventType::EventTag);
    assert_eq!(inserted.event_data, b"EventE");

    // 3. Verify digest (SHA256 of "EventE")
    let expected_digest = Digest::new_sha256(b"EventE");
    assert_eq!(inserted.digests, vec![expected_digest]);
}

#[test]
fn test_added_boot_entry() {
    let good_log = vec![
        moc_boot_event(0x0000, true),
        moc_boot_event(0x0001, true),
        moc_boot_event(0x0002, true),
    ];

    let bad_log = vec![
        moc_boot_event(0x0000, true),
        moc_boot_event(0x0001, true),
        moc_boot_event(0x0003, true),
        moc_boot_event(0x0002, true),
    ];

    // --- Compute LCS ---
    let lcs = compute_lcs(&good_log, &bad_log);

    // --- Find Differences ---
    // Find deletions and insertions
    // Note: We don't care about modifications in this test
    // since we are only adding a new boot entry
    // and not modifying an existing one
    let (deletions, insertions) = collect_diff(&good_log, &bad_log, &lcs);

    // --- Assertions ---
    // 1. No deletions expected
    // We are only adding a new boot entry
    assert!(
        deletions.is_empty(),
        "Unexpected deletions: {:?}",
        deletions
    );
    assert_eq!(insertions.len(), 1);

    // 2. Verify the inserted boot entry
    let inserted = insertions[0];
    assert_eq!(inserted.event_type, TcgTpmEventType::EfiVariableBoot2);
    let efi_var = TcgEfiVariableEvent::try_from(inserted).unwrap();
    assert_eq!(efi_var.unicode_name, "Boot0003");
    assert_eq!(efi_var.variable_data, 0x0003u16.to_le_bytes().to_vec());
}

#[test]
fn test_modified_boot_order() {
    // --- Create Mock Events ---
    // Good BootOrder: [0x0000, 0x0001]
    let good_event = mock_boot_order_event(&[0x0000, 0x0001], true);

    // Bad BootOrder: [0x0001, 0x0000] (modified)
    let bad_event = mock_boot_order_event(&[0x0001, 0x0000], false);

    // Create logs with just the BootOrder event
    let good_log = vec![good_event];
    let bad_log = vec![bad_event];

    // --- Compute LCS ---
    let lcs = compute_lcs(&good_log, &bad_log);

    // LCS should be empty since the events are different
    assert!(lcs.is_empty(), "LCS should detect no common events");

    // --- Find Differences ---
    let (deletions, insertions) = collect_diff(&good_log, &bad_log, &lcs);

    // // --- Detect Modifications ---
    // let mut mods = Vec::new();
    // let del_map: HashMap<_, _> = deletions
    //     .iter()
    //     .filter_map(|e| get_event_semantic_key(e).map(|k| (k, e)))
    //     .collect();

    // for ins in &insertions {
    //     if let Some(key) = get_event_semantic_key(ins) {
    //         if let Some(del_event) = del_map.get(&key) {
    //             mods.push((*del_event, ins));
    //         }
    //     }
    // }

    // // --- Assertions ---
    // // 1. Should detect one modification
    // assert_eq!(mods.len(), 1, "Expected 1 modified event");

    // // 2. Verify the modification details
    // let (old_event, new_event) = mods[0];
    // let old_boot_order = TcgEfiVariableEvent::try_from(*old_event)
    //     .unwrap()
    //     .variable_data
    //     .chunks(2)
    //     .map(|c| u16::from_le_bytes([c[0], c[1]]))
    //     .collect::<Vec<_>>();

    // let new_boot_order = TcgEfiVariableEvent::try_from(*new_event)
    //     .unwrap()
    //     .variable_data
    //     .chunks(2)
    //     .map(|c| u16::from_le_bytes([c[0], c[1]]))
    //     .collect::<Vec<_>>();

    // assert_eq!(old_boot_order, vec![0x0000, 0x0001]);
    // assert_eq!(new_boot_order, vec![0x0001, 0x0000]);

    // // 3. No remaining insertions/deletions
    // assert_eq!(insertions.len() - mods.len(), 0);
    // assert_eq!(deletions.len() - mods.len(), 0);
}

#[test]
fn test_pcr14_file_removed() {
    let old_good_events = vec![moc_pcr14_event("file1", true, None)];
    let new_events = vec![moc_pcr14_event("file1", false, None)];

    let old_good_log = EveTpmLog::from_events(old_good_events);
    let new_log = EveTpmLog::from_events(new_events);

    let (deleted, added, mods) = tpm_log_diff_for_pcr(&old_good_log, &new_log, 14).unwrap();

    assert_eq!(added.len(), 0);
    assert_eq!(deleted.len(), 0);
    assert_eq!(mods.len(), 1);

    let interpretation = interpret_pcr14(deleted, added, mods);

    assert_eq!(interpretation.len(), 1);
    if let InterpretedTpmEvent::ConfigFileModified { file, status } = &interpretation[0] {
        assert_eq!(file, "file1");
        assert_eq!(*status, crate::tpm::diff::ConfigFileStatus::Deleted);
    } else {
        panic!("Unexpected interpretation: {:?}", &interpretation);
    }
}

#[test]
fn test_pcr14_new_file() {
    let old_good_events = vec![moc_pcr14_event("file1", false, None)];
    let new_events = vec![moc_pcr14_event("file1", true, None)];

    let old_good = EveTpmLog::from_events(old_good_events);
    let new = EveTpmLog::from_events(new_events);

    let (deleted, added, mods) = tpm_log_diff_for_pcr(&old_good, &new, 14).unwrap();

    assert_eq!(deleted.len(), 0);
    assert_eq!(added.len(), 0);
    assert_eq!(mods.len(), 1);

    let interpretation = interpret_pcr14(deleted, added, mods);
    assert_eq!(interpretation.len(), 1);

    if let InterpretedTpmEvent::ConfigFileModified { file, status } = &interpretation[0] {
        assert_eq!(file, "file1");
        assert_eq!(*status, crate::tpm::diff::ConfigFileStatus::Added);
    } else {
        panic!("Unexpected interpretation: {:?}", &interpretation);
    }
}

#[test]
fn test_pcr14_new_file_with_hash() {
    let old_good_events = vec![moc_pcr14_event("file1", false, None)];
    let new_events = vec![moc_pcr14_event(
        "file1",
        true,
        Some("61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8"),
    )];

    let old_good = EveTpmLog::from_events(old_good_events);
    let new = EveTpmLog::from_events(new_events);

    let (deleted, added, mods) = tpm_log_diff_for_pcr(&old_good, &new, 14).unwrap();

    assert_eq!(deleted.len(), 0);
    assert_eq!(added.len(), 0);

    let interpretation = interpret_pcr14(deleted, added, mods);

    assert_eq!(interpretation.len(), 1);
    if let InterpretedTpmEvent::ConfigFileModified { file, status } = &interpretation[0] {
        assert_eq!(file, "file1");
        assert_eq!(*status, crate::tpm::diff::ConfigFileStatus::Added);
    } else {
        panic!("Unexpected interpretation: {:?}", &interpretation);
    }
}

#[test]
fn test_pcr14_file_modified() {
    let old_good_events = vec![moc_pcr14_event(
        "file1",
        true,
        Some("61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8"),
    )];
    let new_events = vec![moc_pcr14_event(
        "file1",
        true,
        Some("bb1451b8335cd0ef0f8d6e515154c94d764f1ddd0b247f5e6199ae3b2deec930"),
    )];

    let old_good = EveTpmLog::from_events(old_good_events);
    let new = EveTpmLog::from_events(new_events);

    let (deleted, added, mods) = tpm_log_diff_for_pcr(&old_good, &new, 14).unwrap();

    assert_eq!(added.len(), 0);
    assert_eq!(deleted.len(), 0);
    assert_eq!(mods.len(), 1);

    let interpretation = interpret_pcr14(deleted, added, mods);

    assert_eq!(interpretation.len(), 1);
    if let InterpretedTpmEvent::ConfigFileModified { file, status } = &interpretation[0] {
        assert_eq!(file, "file1");
        assert_eq!(*status, crate::tpm::diff::ConfigFileStatus::Modified);
    } else {
        panic!("Unexpected interpretation: {:?}", &interpretation);
    }
}

#[test]
fn test_pcr14_real_log_1() {
    let old_good_bin =
        std::fs::read("./src/tpm/test_data/pcr8-14/tpm_measurement_seal_success").unwrap();
    let new_bin = std::fs::read("./src/tpm/test_data/pcr8-14/tpm_measurement_unseal_fail").unwrap();

    let old_good_log = EveTpmLog::from_events(TPMLog::from_slice(&old_good_bin).unwrap().events);
    let new_log = EveTpmLog::from_events(TPMLog::from_slice(&new_bin).unwrap().events);

    let (deleted, added, mods) = tpm_log_diff_for_pcr(&old_good_log, &new_log, 14).unwrap();

    assert_eq!(added.len(), 0);
    assert_eq!(deleted.len(), 0);
    assert_eq!(mods.len(), 2);

    let interpretation = interpret_pcr14(deleted, added, mods);

    assert_eq!(interpretation.len(), 2);
    if let InterpretedTpmEvent::ConfigFileModified { file, status } = &interpretation[0] {
        assert_eq!(file, "/config/device.cert.pem");
        assert_eq!(*status, crate::tpm::diff::ConfigFileStatus::Added);
    } else {
        panic!("Unexpected interpretation: {:?}", &interpretation);
    }

    if let InterpretedTpmEvent::ConfigFileModified { file, status } = &interpretation[1] {
        assert_eq!(file, "/config/tpm_credential");
        assert_eq!(*status, crate::tpm::diff::ConfigFileStatus::Added);
    } else {
        panic!("Unexpected interpretation: {:?}", &interpretation);
    }
}

// #[test]
// fn test_log1() {
//     let data_good = std::fs::read("./src/tpm/test_data/tpm_measurement_seal_success").unwrap();
//     let data_bad = std::fs::read("./src/tpm/test_data/tpm_measurement_unseal_fail").unwrap();

//     let good_log = TPMLog::from_slice(&data_good).unwrap();
//     let bad_log = TPMLog::from_slice(&data_bad).unwrap();

//     let good_log = good_log.events_for_pcr_ref(1);
//     let bad_log = bad_log.events_for_pcr_ref(1);

//     // --- Compute LCS ---
//     let lcs = tpm_log_compute_lcs(&good_log, &bad_log);
//     // print lcs
//     for event in &lcs {
//         println!("{:?}", event.event_type);
//     }

//     // --- Find Differences ---
//     let (deletions, insertions) = tpm_log_diff_binary(&good_log, &bad_log, &lcs);

//     // --- Detect Modifications ---
//     let mut mods = Vec::new();
//     let del_map: HashMap<_, _> = deletions
//         .iter()
//         .filter_map(|e| get_event_semantic_key(e).map(|k| (k, e)))
//         .collect();

//     for ins in &insertions {
//         if let Some(key) = get_event_semantic_key(ins) {
//             if let Some(del_event) = del_map.get(&key) {
//                 mods.push((*del_event, ins));
//             }
//         }
//     }

//     panic!();
//     // print insertions
//     // for event in &insertions {
//     //     println!("+ {:?}", event.event_type);
//     // }

//     // // print deletions
//     // for event in &deletions {
//     //     println!("- {:?}", event.event_type);
//     // }

//     // // print modifications
//     // for (old_event, new_event) in &mods {
//     //     println!("M {:?} -> {:?}", old_event.event_type, new_event.event_type);
//     // }
// }

#[test]
fn test_try_from_tpm_event_footfs_event() {
    let tpm_event = mock_tcg_tpm_event(
        13,
        TcgTpmEventType::IPL,
        "squash4 b6dd08d6bc197ea4417bcbc844ecdbe173af97504555d64014380a968aae9c43",
    );
    let rootfs_measurement_event = TpmEvent::try_from_tcg_event(&tpm_event, None).unwrap();

    match rootfs_measurement_event {
        TpmEvent::MeasureRoot { rootfs, hash } => {
            assert_eq!(rootfs, "squash4");
            assert_eq!(
                hash,
                "b6dd08d6bc197ea4417bcbc844ecdbe173af97504555d64014380a968aae9c43"
            );
        }
        _ => panic!("Invalid rootfs event"),
    }
}

#[test]
fn test_try_from_tpm_event_action_config() {
    let tpm_event = mock_tcg_tpm_event(
        14,
        TcgTpmEventType::EfiAction,
        b"file:/config/authorized_keys exist:true \
        content-hash:61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8",
    );

    let action_event = TpmEvent::try_from_tcg_event(&tpm_event, None).unwrap();

    match action_event {
        TpmEvent::MeasureConfig { file, hash, exists } => {
            assert_eq!(file, "/config/authorized_keys");
            assert_eq!(
                hash,
                "61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8"
            );
            assert_eq!(exists, true);
        }
        _ => panic!("Invalid action event"),
    }
}

#[test]
fn test_try_from_tpm_event_action_config_not_exist() {
    let tpm_event = mock_tcg_tpm_event(
        14,
        TcgTpmEventType::EfiAction,
        b"file:/config/authorized_keys exist:false",
    );

    let action_event = TpmEvent::try_from_tcg_event(&tpm_event, None).unwrap();

    match action_event {
        TpmEvent::MeasureConfig { file, hash, exists } => {
            assert_eq!(file, "/config/authorized_keys");
            assert_eq!(hash, "");
            assert_eq!(exists, false);
        }
        _ => panic!("Invalid action event"),
    }
}

#[test]
fn test_try_from_tpm_event_action_config_not_exist_hash() {
    let tpm_event = mock_tcg_tpm_event(
        14,
        TcgTpmEventType::EfiAction,
        b"file:/config/authorized_keys exist:false \
        content-hash:61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8",
    );

    // should fail because hash is not empty
    let action_event = TpmEvent::try_from_tcg_event(&tpm_event, None);
    match action_event {
        Ok(_) => panic!("must fail"),
        Err(e) => assert_eq!(
            e.to_string(),
            "Invalid TpmEvent::MeasureConfig: hash is not empty for exist:false"
        ),
    }
}

#[test]
fn test_try_from_tpm_event_action_config_exist_no_hash() {
    let tpm_event = mock_tcg_tpm_event(
        14,
        TcgTpmEventType::EfiAction,
        b"file:/config/authorized_keys exist:true",
    );

    // should fail because hash is not empty
    let action_event = TpmEvent::try_from_tcg_event(&tpm_event, None).unwrap();
    match action_event {
        TpmEvent::MeasureConfig { file, hash, exists } => {
            assert_eq!(file, "/config/authorized_keys");
            assert_eq!(hash, "");
            assert_eq!(exists, true);
        }
        _ => panic!("Invalid action event"),
    }
}

#[test]
fn test_try_from_grub_event_cmd() {
    let tpm_event = mock_tcg_tpm_event(
        8,
        TcgTpmEventType::IPL,
        b"grub_cmd export dom0_flavor_tweaks",
    );

    let grub_event = TpmEvent::try_from_tcg_event(&tpm_event, None).unwrap();

    match grub_event {
        TpmEvent::GrubCmd { cmd, params } => {
            assert_eq!(cmd, "export");
            assert_eq!(params, "dom0_flavor_tweaks");
        }
        _ => panic!("Invalid grub event"),
    }
}
#[test]
fn test_try_from_grub_event_kernel_cmdline() {
    let tpm_event = mock_tcg_tpm_event(
        8,
        TcgTpmEventType::IPL,
        b"grub_kernel_cmdline /boot/kernel console=ttyS0 console=hvc0 root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 dom0_mem=640M,\
        max:800M dom0_max_vcpus=1 dom0_vcpus_pin eve_mem=520M,max:650M eve_max_vcpus=1 ctrd_mem=320M,max:400M ctrd_max_vcpus=1 \
        change=500 clocksource=tsc clocksource_failover=xen pcie_acs_override=downstream,multifunction crashkernel=2G-16G:128M,16G-128G:256M,128G-:512M \
        rootdelay=3 linuxkit.unified_cgroup_hierarchy=0 panic=120 rfkill.default_state=0 split_lock_detect=off test",
    );

    let grub_event = TpmEvent::try_from_tcg_event(&tpm_event, None).unwrap();

    match grub_event {
        TpmEvent::GrubKernelCmdline(cmd) => {
            assert_eq!(cmd,"/boot/kernel console=ttyS0 console=hvc0 root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 dom0_mem=640M,max:800M \
                dom0_max_vcpus=1 dom0_vcpus_pin eve_mem=520M,max:650M eve_max_vcpus=1 ctrd_mem=320M,max:400M \
                ctrd_max_vcpus=1 change=500 clocksource=tsc clocksource_failover=xen pcie_acs_override=downstream,multifunction \
                crashkernel=2G-16G:128M,16G-128G:256M,128G-:512M rootdelay=3 linuxkit.unified_cgroup_hierarchy=0 panic=120 rfkill.default_state=0 \
                split_lock_detect=off test" );
        }
        _ => panic!("Invalid grub event"),
    }
}

#[test]
fn test_try_from_grub_event_linuxefi() {
    let tpm_event = mock_tcg_tpm_event(
        8,
        TcgTpmEventType::IPL,
        b"grub_linuxefi /boot/vmlinuz-5.4.0-104-generic root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 ro quiet splash vt.handoff=7",
    );

    let grub_event = TpmEvent::try_from_tcg_event(&tpm_event, None).unwrap();

    match grub_event {
        TpmEvent::GrubLinuxEfi(cmd) => {
            assert_eq!(cmd,"/boot/vmlinuz-5.4.0-104-generic root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 ro quiet splash vt.handoff=7" );
        }
        _ => panic!("Invalid grub event"),
    }
}
#[test]
fn test_try_from_grub_event_invalid() {
    let tpm_event = mock_tcg_tpm_event(8, TcgTpmEventType::IPL, b"invalid_event data");

    let grub_event = TpmEvent::try_from_tcg_event(&tpm_event, None);
    match grub_event {
        Ok(_) => panic!("must fail"),
        Err(e) => assert_eq!(e.to_string(), "Invalid grub event type invalid_event"),
    }
}
#[test]
fn test_try_from_grub_event_invalid_pcr() {
    let tpm_event = mock_tcg_tpm_event(
        1,
        TcgTpmEventType::IPL,
        b"grub_cmd export dom0_flavor_tweaks",
    );

    let grub_event = TpmEvent::try_from_tcg_event(&tpm_event, None);
    match grub_event {
        Ok(_) => panic!("must fail"),
        Err(e) => assert_eq!(e.to_string(), "Unsupported event type: IPL for PCR 1"),
    }
}
