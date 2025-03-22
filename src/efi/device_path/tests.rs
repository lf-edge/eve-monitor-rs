// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::efi::device_path::media::{PartitionSignature, PartitionType};

use super::DevicePath;

#[test]
fn text_device_path_efi_spec_p312() {
    let path = DevicePath::new()
        .acpi_acpi(0x0A03, 0x0)
        .hw_pci(0, 0x19)
        .msg_mac_addr("AA:11:22:33:44:55".parse().unwrap(), 0x1)
        .msg_ipv4(
            "192.168.0.1".parse().unwrap(),
            "192.168.0.100".parse().unwrap(),
            0,
            3260,
            true,
            6,
            "1.1.1.1".parse().unwrap(),
            "255.255.255.0".parse().unwrap(),
        )
        .msg_i_scsi(
            0x800,
            0x1,
            0x0,
            "iqn.1991-05.com.microsoft:iscsitarget-iscsidisk-target",
        )
        .media_hdd(
            1,
            0x22,
            0x2710000,
            PartitionSignature::Guid(uuid::uuid!("15E39A00-1DD2-1000-8D7F-00A0C92408FC")),
            PartitionType::Gpt,
        );
    assert_eq!(
        path.nodes[0].to_bytes(),
        vec![0x02, 0x01, 0x0C, 0x0, 0xd0, 0x41, 0x03, 0x0a, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        path.nodes[1].to_bytes(),
        vec![0x01, 0x01, 0x06, 0x0, 0x00, 0x19]
    );
    assert_eq!(
        path.nodes[2].to_bytes(),
        vec![
            0x03, 0x0b, 0x25, 0x00, //size
            0xAA, 0x11, 0x22, 0x33, 0x44, 0x55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, // padded mac address
            0x1
        ]
    );
    assert_eq!(
        path.nodes[3].to_bytes(),
        vec![
            0x03, 0x0c, 0x1b, 0x0, 0xc0, 0xA8, 0x00, 0x01, 0xC0, 0xA8, 0x00, 0x64, 0x00, 0x00,
            0xbc, 0x0c, 0x6, 0x0, 1, 0x1, 0x1, 0x1, 0x1, 0xff, 0xff, 0xff, 0
        ]
    );
    assert_eq!(
        path.nodes[4].to_bytes(),
        vec![
            0x03, 0x13, 0x49, 0x0, 0x00, 0x0, 0x0, 0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0x0, 0x69,
            0x71, 0x6E, 0x2E, 0x31, 0x39, 0x39, 0x31, 0x2D, 0x30, 0x35, 0x2E, 0x63, 0x6F, 0x6D,
            0x2E, 0x6D, 0x69, 0x63, 0x72, 0x6F, 0x73, 0x6F, 0x66, 0x74, 0x3a, 0x69, 0x73, 0x63,
            0x73, 0x69, 0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x2D, 0x69, 0x73, 0x63, 0x73, 0x69,
            0x64, 0x69, 0x73, 0x6B, 0x2D, 0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x0
        ]
    );
    assert_eq!(
        path.nodes[5].to_bytes(),
        vec![
            0x04, 0x1, 0x2A, 0x00, 0x1, 0x00, 0x00, 0x00, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x71, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x9A, 0xE3, 0x15, 0xD2,
            0x1D, 0x00, 0x10, 0x8D, 0x7F, 0x00, 0xA0, 0xC9, 0x24, 0x08, 0xFC, 0x2, 0x2
        ]
    );
}

#[test]
fn test_device_path_display() {
    let path = DevicePath::new()
        .acpi_acpi(0x0A03, 0x0)
        .hw_pci(0, 0x19)
        .msg_mac_addr("AA:11:22:33:44:55".parse().unwrap(), 0x1)
        .msg_ipv4(
            "192.168.0.1".parse().unwrap(),
            "192.168.0.100".parse().unwrap(),
            0,
            3260,
            true,
            6,
            "1.1.1.1".parse().unwrap(),
            "255.255.255.0".parse().unwrap(),
        )
        .msg_i_scsi(
            0x800,
            0x1,
            0x0,
            "iqn.1991-05.com.microsoft:iscsitarget-iscsidisk-target",
        )
        .media_hdd(
            1,
            0x22,
            0x2710000,
            PartitionSignature::Guid(uuid::uuid!("15E39A00-1DD2-1000-8D7F-00A0C92408FC")),
            PartitionType::Gpt,
        );
    let display = path.display(false);
    println!("{}", display);
}
