// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use uuid::Uuid;

use super::tpmlog::{EvePcrIndex, TpmEvent, TpmEventType};

#[derive(Debug)]
pub struct EFIVariableBootEvent {
    pub vendor_guid: Uuid,
    pub unicode_name: String,
    pub variable_data: Vec<u8>,
}

// corresponds to EV_IPL event for PCR 8
pub enum GrubEvent {
    Cmd(String),
    KernelCmdLine(String),
    LinuxEfi(String),
}

impl TryFrom<&TpmEvent> for GrubEvent {
    type Error = anyhow::Error;

    fn try_from(value: &TpmEvent) -> std::result::Result<Self, Self::Error> {
        if value.event_type != TpmEventType::IPL {
            return Err(anyhow::anyhow!(
                "Invalid event type for grub event {}",
                value.event_type
            ));
        }

        if value.pcr_index != EvePcrIndex::GrubPcr as u32 {
            return Err(anyhow::anyhow!(
                "Invalid PCR index for grub event {}",
                value.pcr_index
            ));
        }

        let event_data = std::str::from_utf8(&value.event_data)
            .context("Error converting event data to utf-8 string")?;

        // split by first space and keep both parts
        let event_data = event_data.splitn(2, ' ').collect::<Vec<&str>>();

        if event_data.len() != 2 {
            return Err(anyhow::anyhow!("Invalid event data for grub event"));
        }

        let event_type = event_data.get(0).unwrap().to_string();
        let event_data = event_data.get(1).unwrap().to_string();

        match event_type.as_str() {
            "grub_cmd" => Ok(GrubEvent::Cmd(event_data)),
            "grub_kernel_cmdline" => Ok(GrubEvent::KernelCmdLine(event_data)),
            "grub_linuxefi" => Ok(GrubEvent::LinuxEfi(event_data)),
            _ => Err(anyhow::anyhow!("Invalid grub event type {}", event_type)),
        }
    }
}

// corresponds to EV_IPL event for PCR 13
pub struct RootFsMeasurementEvent {
    pub rootfs: String,
    pub hash: String,
}

impl TryFrom<&TpmEvent> for RootFsMeasurementEvent {
    type Error = anyhow::Error;

    fn try_from(value: &TpmEvent) -> std::result::Result<Self, Self::Error> {
        if value.event_type != TpmEventType::IPL {
            return Err(anyhow::anyhow!(
                "Invalid event type for rootfs measurement event {}",
                value.event_type
            ));
        }

        if value.pcr_index != EvePcrIndex::RootFsPcr as u32 {
            return Err(anyhow::anyhow!(
                "Invalid PCR index for rootfs measurement event {}",
                value.pcr_index
            ));
        }

        // treat event data as utf-8 string
        let event_data = std::str::from_utf8(&value.event_data)
            .context("Error converting event data to utf-8 string")?;
        // split by space
        let mut parts = event_data.split_whitespace();
        // first part is rootfs type
        let rootfs = parts
            .next()
            .context("Error parsing rootfs type from event data")?;
        // second part is a sha256 hash
        let hash = parts.next().context("Error parsing hash from event data")?;
        Ok(Self {
            rootfs: rootfs.to_string(),
            hash: hash.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::tpm::tpmlog::Digest;

    #[test]
    fn test_try_from_tpm_event_footfs_event() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::IPL,
            event_data: b"squash4 b6dd08d6bc197ea4417bcbc844ecdbe173af97504555d64014380a968aae9c43"
                .to_vec(),
            pcr_index: EvePcrIndex::RootFsPcr as u32,
            digests: vec![Digest::new_sha256(
                &b"squash4 b6dd08d6bc197ea4417bcbc844ecdbe173af97504555d64014380a968aae9c43"
                    .to_vec(),
            )],
        };
        let rootfs_measurement_event = RootFsMeasurementEvent::try_from(&tpm_event).unwrap();
        assert_eq!(rootfs_measurement_event.rootfs, "squash4");
        assert_eq!(
            rootfs_measurement_event.hash,
            "b6dd08d6bc197ea4417bcbc844ecdbe173af97504555d64014380a968aae9c43"
        );
    }

    #[test]
    fn test_try_from_tpm_event_action_config() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::Action,
            event_data: b"file:/config/authorized_keys exist:true content-hash:61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8"
                .to_vec(),
            pcr_index: EvePcrIndex::ConfigPcr as u32,
            digests: vec![Digest::new_sha256(
                &b"file:/config/authorized_keys exist:true content-hash:61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8"
                    .to_vec(),
            )],
        };

        let action_event = ActionEvent::try_from(&tpm_event).unwrap();

        match action_event {
            ActionEvent::MeasureConfig { file, hash, exists } => {
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
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::Action,
            event_data: b"file:/config/authorized_keys exist:false".to_vec(),
            pcr_index: EvePcrIndex::ConfigPcr as u32,
            digests: vec![Digest::new_sha256(
                &b"file:/config/authorized_keys exist:false".to_vec(),
            )],
        };

        let action_event = ActionEvent::try_from(&tpm_event).unwrap();

        match action_event {
            ActionEvent::MeasureConfig { file, hash, exists } => {
                assert_eq!(file, "/config/authorized_keys");
                assert_eq!(hash, "");
                assert_eq!(exists, false);
            }
            _ => panic!("Invalid action event"),
        }
    }

    #[test]
    fn test_try_from_tpm_event_action_config_not_exist_hash() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::Action,
            event_data: b"file:/config/authorized_keys exist:false content-hash:61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8".to_vec(),
            pcr_index: EvePcrIndex::ConfigPcr as u32,
            digests: vec![Digest::new_sha256(
                &b"file:/config/authorized_keys exist:false content-hash:61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8".to_vec(),
            )],
        };

        // should fail because hash is not empty
        let action_event = ActionEvent::try_from(&tpm_event);
        match action_event {
            Ok(_) => panic!("must fail"),
            Err(e) => assert_eq!(
                e.to_string(),
                "Invalid TpmEventType::Action: hash is not empty"
            ),
        }
    }

    #[test]
    fn test_try_from_tpm_event_action_config_exist_no_hash() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::Action,
            event_data: b"file:/config/authorized_keys exist:true".to_vec(),
            pcr_index: EvePcrIndex::ConfigPcr as u32,
            digests: vec![Digest::new_sha256(
                &b"file:/config/authorized_keys exist:true".to_vec(),
            )],
        };

        // should fail because hash is not empty
        let action_event = ActionEvent::try_from(&tpm_event).unwrap();
        match action_event {
            ActionEvent::MeasureConfig { file, hash, exists } => {
                assert_eq!(file, "/config/authorized_keys");
                assert_eq!(hash, "");
                assert_eq!(exists, true);
            }
            _ => panic!("Invalid action event"),
        }
    }

    #[test]
    fn test_try_from_grub_event_cmd() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::IPL,
            event_data: b"grub_cmd export dom0_flavor_tweaks".to_vec(),
            pcr_index: EvePcrIndex::GrubPcr as u32,
            digests: vec![Digest::new_sha256(
                &b"grub_cmd export dom0_flavor_tweaks".to_vec(),
            )],
        };

        let grub_event = GrubEvent::try_from(&tpm_event).unwrap();

        match grub_event {
            GrubEvent::Cmd(cmd) => {
                assert_eq!(cmd, "export dom0_flavor_tweaks");
            }
            _ => panic!("Invalid grub event"),
        }
    }
    #[test]
    fn test_try_from_grub_event_kernel_cmdline() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::IPL,
            event_data: b"grub_kernel_cmdline /boot/kernel console=ttyS0 console=hvc0 root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 dom0_mem=640M,max:800M dom0_max_vcpus=1 dom0_vcpus_pin eve_mem=520M,max:650M eve_max_vcpus=1 ctrd_mem=320M,max:400M ctrd_max_vcpus=1 change=500 clocksource=tsc clocksource_failover=xen pcie_acs_override=downstream,multifunction crashkernel=2G-16G:128M,16G-128G:256M,128G-:512M rootdelay=3 linuxkit.unified_cgroup_hierarchy=0 panic=120 rfkill.default_state=0 split_lock_detect=off test".to_vec(),
            pcr_index: EvePcrIndex::GrubPcr as u32,
            digests: vec![Digest::new_sha256(&b"grub_kernel_cmdline /boot/kernel console=ttyS0 console=hvc0 root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 dom0_mem=640M,max:800M dom0_max_vcpus=1 dom0_vcpus_pin eve_mem=520M,max:650M eve_max_vcpus=1 ctrd_mem=320M,max:400M ctrd_max_vcpus=1 change=500 clocksource=tsc clocksource_failover=xen pcie_acs_override=downstream,multifunction crashkernel=2G-16G:128M,16G-128G:256M,128G-:512M rootdelay=3 linuxkit.unified_cgroup_hierarchy=0 panic=120 rfkill.default_state=0 split_lock_detect=off test".to_vec())],
        };

        let grub_event = GrubEvent::try_from(&tpm_event).unwrap();

        match grub_event {
            GrubEvent::KernelCmdLine(cmd) => {
                assert_eq!(cmd,"/boot/kernel console=ttyS0 console=hvc0 root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 dom0_mem=640M,max:800M dom0_max_vcpus=1 dom0_vcpus_pin eve_mem=520M,max:650M eve_max_vcpus=1 ctrd_mem=320M,max:400M ctrd_max_vcpus=1 change=500 clocksource=tsc clocksource_failover=xen pcie_acs_override=downstream,multifunction crashkernel=2G-16G:128M,16G-128G:256M,128G-:512M rootdelay=3 linuxkit.unified_cgroup_hierarchy=0 panic=120 rfkill.default_state=0 split_lock_detect=off test" );
            }
            _ => panic!("Invalid grub event"),
        }
    }

    #[test]
    fn test_try_from_grub_event_linuxefi() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::IPL,
            event_data: b"grub_linuxefi /boot/vmlinuz-5.4.0-104-generic root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 ro quiet splash vt.handoff=7".to_vec(),
            pcr_index: EvePcrIndex::GrubPcr as u32,
            digests: vec![Digest::new_sha256(&b"grub_linuxefi /boot/vmlinuz-5.4.0-104-generic root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 ro quiet splash vt.handoff=7".to_vec())],
        };

        let grub_event = GrubEvent::try_from(&tpm_event).unwrap();

        match grub_event {
            GrubEvent::LinuxEfi(cmd) => {
                assert_eq!(cmd,"/boot/vmlinuz-5.4.0-104-generic root=PARTUUID=ad6871ee-31f9-4cf3-9e09-6f7a25c30052 ro quiet splash vt.handoff=7" );
            }
            _ => panic!("Invalid grub event"),
        }
    }
    #[test]
    fn test_try_from_grub_event_invalid() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::IPL,
            event_data: b"invalid_event data".to_vec(),
            pcr_index: EvePcrIndex::GrubPcr as u32,
            digests: vec![Digest::new_sha256(&b"invalid_event".to_vec())],
        };

        let grub_event = GrubEvent::try_from(&tpm_event);
        match grub_event {
            Ok(_) => panic!("must fail"),
            Err(e) => assert_eq!(e.to_string(), "Invalid grub event type invalid_event"),
        }
    }
    #[test]
    fn test_try_from_grub_event_invalid_pcr() {
        use super::*;
        let tpm_event = TpmEvent {
            event_type: TpmEventType::IPL,
            event_data: b"grub_cmd export dom0_flavor_tweaks".to_vec(),
            pcr_index: 1,
            digests: vec![Digest::new_sha256(
                &b"grub_cmd export dom0_flavor_tweaks".to_vec(),
            )],
        };

        let grub_event = GrubEvent::try_from(&tpm_event);
        match grub_event {
            Ok(_) => panic!("must fail"),
            Err(e) => assert_eq!(e.to_string(), "Invalid PCR index for grub event 1"),
        }
    }
}

pub enum ActionEvent {
    EnterBiosSetup, // corresponds to EV_EFI_ACTION event for PCR 1
    // corresponds to EV_EFI_ACTION event for PCR 14
    MeasureConfig {
        file: String,
        hash: String,
        exists: bool,
    },
    Action(String),
}

impl TryFrom<&TpmEvent> for ActionEvent {
    type Error = anyhow::Error;

    fn try_from(value: &TpmEvent) -> std::result::Result<Self, Self::Error> {
        if value.event_type != TpmEventType::Action {
            return Err(anyhow::anyhow!(
                "Invalid event type for action event {}",
                value.event_type
            ));
        }

        let evetnt_value = std::str::from_utf8(&value.event_data)
            .context("Error converting event data to utf-8 string")?;

        match value.pcr_index {
            // e.g file:/config/authorized_keys exist:true content-hash:61e3c4e3aaee97c87c12d4dfbd699b11007e3a5900b02d53f18d978f31cfcaf8
            // content-hash can be omitted if file does not exist
            14 => {
                let re =
                    regex::Regex::new(r"file:(\S+) exist:(true|false)(?: content-hash:(\S+))?")?;

                let captures = re
                    .captures(evetnt_value)
                    .context("Error parsing action event")?;
                let file = captures.get(1).context("Error parsing 'file:'")?.as_str();
                let exists = captures.get(2).context("Error parsing 'exists:'")?.as_str() == "true";
                let hash = captures.get(3).map(|m| m.as_str()).unwrap_or_default();

                if !exists && !hash.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Invalid TpmEventType::Action: hash is not empty"
                    ));
                }

                Ok(ActionEvent::MeasureConfig {
                    file: file.to_string(),
                    hash: hash.to_string(),
                    exists,
                })
            }
            1 | 3 if evetnt_value == "Entering ROM Based Setup" => Ok(ActionEvent::EnterBiosSetup),
            1 | 3 | 4 | 5 | 7 => Ok(ActionEvent::Action(evetnt_value.to_string())),
            _ => Err(anyhow::anyhow!(
                "Invalid PCR index for TpmEventType::Action {}",
                value.pcr_index
            )),
        }
    }
}

impl EFIVariableBootEvent {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        // Read vendor GUID (16 bytes)
        let mut vendor_guid = [0u8; 16];
        cursor.read_exact(&mut vendor_guid)?;

        // convert to GUID
        let vendor_guid = Uuid::from_bytes_le(vendor_guid);

        // Read the UTF-16LE encoded name length in characters (4 bytes)
        let name_length_bytes = cursor.read_u64::<LittleEndian>()? * 2;
        // Read the variable data length in bytes (4 bytes)
        let data_length_bytes = cursor.read_u64::<LittleEndian>()?;

        // Read the UTF-16LE encoded name
        let mut name_bytes = vec![0u8; name_length_bytes as usize];
        cursor
            .read_exact(&mut name_bytes)
            .context("reading variable name")?;

        let name_utf16: Vec<u16> = name_bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        let unicode_name =
            String::from_utf16(&name_utf16).context("Error converting UTF-16 to String")?;

        let mut variable_data = vec![0u8; data_length_bytes as usize];
        cursor.read_exact(&mut variable_data)?;

        Ok(Self {
            vendor_guid,
            unicode_name,
            variable_data,
        })
    }

    // function to serialize the data to [u8]
    // used only for test
    #[cfg(test)]
    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(&self.vendor_guid.to_bytes_le());
        data.extend_from_slice(&(self.unicode_name.len() as u64).to_le_bytes());
        data.extend_from_slice(&(self.variable_data.len() as u64).to_le_bytes());
        data.extend_from_slice(
            &self
                .unicode_name
                .encode_utf16()
                .flat_map(|c| c.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
        data.extend_from_slice(&self.variable_data);
        data
    }
}
