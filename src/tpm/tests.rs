// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

use crate::tpm::{
    diff::{get_event_key, tpm_log_compute_lcs, tpm_log_diff_binary},
    events::EFIVariableBootEvent,
    tpmlog::{Digest, TPMLog, TpmEvent, TpmEventType},
};

// Helper to create EFI variable boot events
fn mock_boot_order_event(order: &[u16], is_good: bool) -> TpmEvent {
    let efi_var = EFIVariableBootEvent {
        vendor_guid: Uuid::parse_str("8BE4DF61-93CA-11D2-AA0D-00E098032B8C").unwrap(), // EFI_GLOBAL_VARIABLE_GUID
        unicode_name: "BootOrder".to_string(),
        variable_data: order
            .iter()
            .flat_map(|v| v.to_le_bytes().to_vec())
            .collect(),
    };

    let event_data = efi_var.serialize();

    TpmEvent {
        pcr_index: 1,
        event_type: if is_good {
            TpmEventType::EfiVariableBoot
        } else {
            TpmEventType::EfiVariableBoot2
        },
        digests: vec![Digest::new_sha256(&event_data)],
        event_data,
    }
}

fn moc_boot_event(index: u16) -> TpmEvent {
    let efi_var = EFIVariableBootEvent {
        vendor_guid: Uuid::parse_str("8BE4DF61-93CA-11D2-AA0D-00E098032B8C").unwrap(), // EFI_GLOBAL_VARIABLE_GUID
        unicode_name: format!("Boot{:04X}", index),
        variable_data: index.to_le_bytes().to_vec(),
    };

    let event_data = efi_var.serialize();

    TpmEvent {
        pcr_index: 1,
        event_type: TpmEventType::EfiVariableBoot,
        digests: vec![Digest::new_sha256(&event_data)],
        event_data,
    }
}

// Helper to create mock events
fn mock_event(pcr: u32, event_type: TpmEventType, data: &str) -> TpmEvent {
    TpmEvent {
        pcr_index: pcr,
        event_type,
        digests: vec![Digest::new_sha256(data.as_bytes())], // Fixed digests
        event_data: data.as_bytes().to_vec(),
    }
}

// fn get_event_key(event: &TpmEvent) -> Option<String> {
//     match event.event_type {
//         TpmEventType::EfiVariableBoot | TpmEventType::EfiVariableBoot2 => {
//             // Try to parse as EFI variable
//             let efi_var = EFIVariableBootEvent::parse(&event.event_data).ok()?;
//             Some(format!("EFIVar:{}", efi_var.unicode_name))
//         }
//         TpmEventType::Action => {
//             // Use event data as key
//             Some(String::from_utf8(event.event_data.clone()).ok()?)
//         }
//         _ => Some(format!("{}", event.event_type)),
//     }
// }

#[test]
fn test_tpmlog() -> Result<()> {
    let data =
        std::fs::read("./src/tpm/test_data/add-uefi-shell/tpm_measurement_seal_success").unwrap();
    let log = TPMLog::from_slice(&data).unwrap();
    println!();
    for event in log.events {
        if event.pcr_index == 1 {
            println!("PCR: {} {:?}", event.pcr_index, event.event_type);
            if event.event_type == TpmEventType::EfiVariableBoot {
                // convert event data to EFIVariableBootEvent
                let _efi_event = EFIVariableBootEvent::parse(&event.event_data)?;
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
        mock_event(1, TpmEventType::NoAction, "EventA"),
        mock_event(1, TpmEventType::PostCode, "EventB"),
        mock_event(1, TpmEventType::Separator, "EventC"),
        mock_event(1, TpmEventType::Action, "EventD"),
    ];

    // "Bad" log events: A, B, *E*, C, D (E inserted)
    let bad_events = vec![
        mock_event(1, TpmEventType::NoAction, "EventA"),
        mock_event(1, TpmEventType::PostCode, "EventB"),
        mock_event(1, TpmEventType::EventTag, "EventE"), // Inserted
        mock_event(1, TpmEventType::Separator, "EventC"),
        mock_event(1, TpmEventType::Action, "EventD"),
    ];

    // --- Compute LCS ---
    let lcs = tpm_log_compute_lcs(
        &good_events.iter().collect::<Vec<_>>(),
        &bad_events.iter().collect::<Vec<_>>(),
    );

    // --- Verify LCS ---
    // Expected LCS: A, B, C, D (excluding inserted E)
    assert_eq!(lcs.len(), 4);
    assert_eq!(lcs[0].event_data, b"EventA");
    assert_eq!(lcs[1].event_data, b"EventB");
    assert_eq!(lcs[2].event_data, b"EventC");
    assert_eq!(lcs[3].event_data, b"EventD");

    // --- Find Differences ---
    let (deletions, insertions) = tpm_log_diff_binary(
        &good_events.iter().collect::<Vec<_>>(),
        &bad_events.iter().collect::<Vec<_>>(),
        &lcs,
    );

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
    assert_eq!(inserted.event_type, TpmEventType::EventTag);
    assert_eq!(inserted.event_data, b"EventE");

    // 3. Verify digest (SHA256 of "EventE")
    let expected_digest = Digest::new_sha256(b"EventE");
    assert_eq!(inserted.digests, vec![expected_digest]);
}

#[test]
fn test_added_boot_entry() {
    let good_log = vec![
        moc_boot_event(0x0000),
        moc_boot_event(0x0001),
        moc_boot_event(0x0002),
    ];

    let bad_log = vec![
        moc_boot_event(0x0000),
        moc_boot_event(0x0001),
        moc_boot_event(0x0003),
        moc_boot_event(0x0002),
    ];

    // --- Compute LCS ---
    let lcs = tpm_log_compute_lcs(
        &good_log.iter().collect::<Vec<_>>(),
        &bad_log.iter().collect::<Vec<_>>(),
    );

    // --- Find Differences ---
    // Find deletions and insertions
    // Note: We don't care about modifications in this test
    // since we are only adding a new boot entry
    // and not modifying an existing one
    let (deletions, insertions) = tpm_log_diff_binary(
        &good_log.iter().collect::<Vec<_>>(),
        &bad_log.iter().collect::<Vec<_>>(),
        &lcs,
    );

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
    let inserted = &insertions[0];
    assert_eq!(inserted.event_type, TpmEventType::EfiVariableBoot);
    let efi_var = EFIVariableBootEvent::parse(&inserted.event_data).unwrap();
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
    let good_log = vec![&good_event];
    let bad_log = vec![&bad_event];

    // --- Compute LCS ---
    let lcs = tpm_log_compute_lcs(&good_log, &bad_log);

    // LCS should be empty since the events are different
    assert!(lcs.is_empty(), "LCS should detect no common events");

    // --- Find Differences ---
    let (deletions, insertions) = tpm_log_diff_binary(&good_log, &bad_log, &lcs);

    // --- Detect Modifications ---
    let mut mods = Vec::new();
    let del_map: HashMap<_, _> = deletions
        .iter()
        .filter_map(|e| get_event_key(e).map(|k| (k, e)))
        .collect();

    for ins in &insertions {
        if let Some(key) = get_event_key(ins) {
            if let Some(del_event) = del_map.get(&key) {
                mods.push((*del_event, ins));
            }
        }
    }

    // --- Assertions ---
    // 1. Should detect one modification
    assert_eq!(mods.len(), 1, "Expected 1 modified event");

    // 2. Verify the modification details
    let (old_event, new_event) = mods[0];
    let old_boot_order = EFIVariableBootEvent::parse(&old_event.event_data)
        .unwrap()
        .variable_data
        .chunks(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<_>>();

    let new_boot_order = EFIVariableBootEvent::parse(&new_event.event_data)
        .unwrap()
        .variable_data
        .chunks(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<_>>();

    assert_eq!(old_boot_order, vec![0x0000, 0x0001]);
    assert_eq!(new_boot_order, vec![0x0001, 0x0000]);

    // 3. No remaining insertions/deletions
    assert_eq!(insertions.len() - mods.len(), 0);
    assert_eq!(deletions.len() - mods.len(), 0);
}

#[test]
fn test_log1() {
    let data = std::fs::read("./src/tpm/test_data/tpm_measurement_seal_success").unwrap();
    let data = std::fs::read("./src/tpm/test_data/tpm_measurement_unseal_fail").unwrap();

    let good_log = TPMLog::from_slice(&data).unwrap();
    let bad_log = TPMLog::from_slice(&data).unwrap();

    let good_log = good_log.events_for_pcr_ref(1);
    let bad_log = bad_log.events_for_pcr_ref(1);

    // --- Compute LCS ---
    let lcs = tpm_log_compute_lcs(&good_log, &bad_log);
    // print lcs
    for event in &lcs {
        println!("{:?}", event.event_type);
    }

    // --- Find Differences ---
    let (deletions, insertions) = tpm_log_diff_binary(&good_log, &bad_log, &lcs);

    // --- Detect Modifications ---
    let mut mods = Vec::new();
    let del_map: HashMap<_, _> = deletions
        .iter()
        .filter_map(|e| get_event_key(e).map(|k| (k, e)))
        .collect();

    for ins in &insertions {
        if let Some(key) = get_event_key(ins) {
            if let Some(del_event) = del_map.get(&key) {
                mods.push((*del_event, ins));
            }
        }
    }

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
}
