// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use macaddr::MacAddr;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::{
    io::Read,
    net::{Ipv4Addr, Ipv6Addr},
};

use super::{
    traits::{DevicePathReadEx, DevicePathWriteEx, NodeExpectedLength},
    Node, NodeTypeValidator, PathNodeTrait,
};

#[cfg(test)]
use super::DevicePathType;

#[derive(Debug, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DevicePathSubTypeMessaging {
    Atapi = 0x1,
    Scsi = 0x2,
    FiberChannel = 0x3,
    FiberChannelEx = 21,
    //IEEE1394 = 0x4,
    Usb = 0x5,
    Sata = 18,
    UsbWwid = 0x10,
    Lun = 17,
    UsbClass = 15,
    I2O = 6,
    MacAddr = 11,
    IpV4 = 12,
    IpV6 = 13,
    IScsi = 19,
    Vlan = 20,
    // InfinitiBand = 9, 48
    // Uart = 14, 19
    // Vendor = 10, >=20
    // Following 2 are Vendor structs
    // UartFlowControl = 10, 24, DEVICE_PATH_MESSAGING_UART_FLOW_CONTROL
    // SAS = 10, 44, d487ddb4-008b-11d9-afdc-001083ffca4d
    // -- end of vendor structs --
    // SasEx = 22, 32
    Nvme = 23,
    Uri = 24,
    // Ufs = 25, 6
    Sd = 26,
    // Bluetooth = 27, 10
    // Wireless = 28, 36
    EMMC = 29,
    #[num_enum(catch_all)]
    Unknown(u8),
}

impl NodeTypeValidator for DevicePathSubTypeMessaging {
    fn expected_length(&self) -> NodeExpectedLength {
        match self {
            DevicePathSubTypeMessaging::Atapi => NodeExpectedLength::Exact(8),
            DevicePathSubTypeMessaging::Scsi => NodeExpectedLength::Exact(8),
            DevicePathSubTypeMessaging::FiberChannel => NodeExpectedLength::Exact(24),
            DevicePathSubTypeMessaging::FiberChannelEx => NodeExpectedLength::Exact(24),
            //DevicePathSubTypeMessaging::IEEE1394 => NodeExpectedLength::Exact(16),
            DevicePathSubTypeMessaging::Usb => NodeExpectedLength::Exact(6),
            DevicePathSubTypeMessaging::Sata => NodeExpectedLength::Exact(10),
            DevicePathSubTypeMessaging::UsbWwid => NodeExpectedLength::Min(10),
            DevicePathSubTypeMessaging::Lun => NodeExpectedLength::Exact(5),
            DevicePathSubTypeMessaging::UsbClass => NodeExpectedLength::Exact(11),
            DevicePathSubTypeMessaging::I2O => NodeExpectedLength::Exact(8),
            DevicePathSubTypeMessaging::MacAddr => NodeExpectedLength::Exact(37),
            DevicePathSubTypeMessaging::IpV4 => NodeExpectedLength::Exact(27),
            DevicePathSubTypeMessaging::IpV6 => NodeExpectedLength::Exact(60),
            DevicePathSubTypeMessaging::IScsi => NodeExpectedLength::Min(38),
            DevicePathSubTypeMessaging::Vlan => NodeExpectedLength::Exact(6),
            DevicePathSubTypeMessaging::Nvme => NodeExpectedLength::Exact(16),
            DevicePathSubTypeMessaging::Uri => NodeExpectedLength::Min(4),
            DevicePathSubTypeMessaging::Sd => NodeExpectedLength::Exact(5),
            DevicePathSubTypeMessaging::EMMC => NodeExpectedLength::Exact(5),
            DevicePathSubTypeMessaging::Unknown(_) => NodeExpectedLength::Min(4),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum MessagingNode {
    Atapi {
        primary: bool,
        slave: bool,
        lun: u16,
    },
    Scsi {
        target: u16,
        lun: u16,
    },
    FiberChannel {
        wwn: u64,
        lun: u64,
    },
    FiberChannelEx {
        //FIXME: must be 8 byte arrays
        wwn: u64,
        lun: u64,
    },
    Sata {
        hba_port: u16,
        port_multiplier_port: u16,
        lun: u16,
    },
    Usb {
        parent_port_number: u8,
        usb_interface: u8,
    },
    UsbWwid {
        interface_number: u16,
        vendor_id: u16,
        product_id: u16,
        serial: Vec<u8>,
    },
    Lun(u8),
    UsbClass {
        vendor_id: u16,
        product_id: u16,
        device_class: u8,
        device_subclass: u8,
        device_protocol: u8,
    },
    MacAddr {
        mac_addr: MacAddr,
        if_type: u8,
    },
    IpV4 {
        local_ip: Ipv4Addr,
        remote_ip: Ipv4Addr,
        local_port: u16,
        remote_port: u16,
        protocol: u16,
        is_static: bool,
        gw: Ipv4Addr,
        mask: Ipv4Addr,
    },
    IpV6 {
        local_ip: Ipv6Addr,
        remote_ip: Ipv6Addr,
        local_port: u16,
        remote_port: u16,
        protocol: u16,
        origin: u8,
        prefix_len: u8,
        gw: Ipv6Addr,
    },
    Vlan {
        vlan_id: u16,
    },
    IScsi {
        protocol: u16,
        options: u16,
        lun: u64,
        group_tag: u16,
        target: String,
    },
    Sd {
        slot: u8,
    },
    EMMC {
        slot: u8,
    },
    Nvme {
        namespace_id: u32,
        namespace_uuid: u64,
    },
    I2O {
        tid: u32,
    },
    Uri {
        uri: String,
    },
    Unknown(Node),
}

impl PathNodeTrait for MessagingNode {
    type Subtype = DevicePathSubTypeMessaging;

    fn get_generic_name(&self) -> &'static str {
        "MessagingPath"
    }

    #[cfg(test)]
    fn get_efi_type(&self) -> DevicePathType {
        DevicePathType::Messaging
    }

    fn get_efi_sub_type(&self) -> Self::Subtype {
        match self {
            MessagingNode::Atapi { .. } => DevicePathSubTypeMessaging::Atapi,
            MessagingNode::Scsi { .. } => DevicePathSubTypeMessaging::Scsi,
            MessagingNode::FiberChannel { .. } => DevicePathSubTypeMessaging::FiberChannel,
            MessagingNode::FiberChannelEx { .. } => DevicePathSubTypeMessaging::FiberChannelEx,
            MessagingNode::Sata { .. } => DevicePathSubTypeMessaging::Sata,
            MessagingNode::Usb { .. } => DevicePathSubTypeMessaging::Usb,
            MessagingNode::UsbWwid { .. } => DevicePathSubTypeMessaging::UsbWwid,
            MessagingNode::Lun(_) => DevicePathSubTypeMessaging::Lun,
            MessagingNode::UsbClass { .. } => DevicePathSubTypeMessaging::UsbClass,
            MessagingNode::MacAddr { .. } => DevicePathSubTypeMessaging::MacAddr,
            MessagingNode::IpV4 { .. } => DevicePathSubTypeMessaging::IpV4,
            MessagingNode::IpV6 { .. } => DevicePathSubTypeMessaging::IpV6,
            MessagingNode::Vlan { .. } => DevicePathSubTypeMessaging::Vlan,
            MessagingNode::IScsi { .. } => DevicePathSubTypeMessaging::IScsi,
            MessagingNode::Sd { .. } => DevicePathSubTypeMessaging::Sd,
            MessagingNode::EMMC { .. } => DevicePathSubTypeMessaging::EMMC,
            MessagingNode::Nvme { .. } => DevicePathSubTypeMessaging::Nvme,
            MessagingNode::I2O { .. } => DevicePathSubTypeMessaging::I2O,
            MessagingNode::Uri { .. } => DevicePathSubTypeMessaging::Uri,
            MessagingNode::Unknown(node) => DevicePathSubTypeMessaging::Unknown(node.node_sub_type),
        }
    }
    fn display(&self, display_only: bool) -> String {
        match self {
            MessagingNode::Atapi {
                primary,
                slave,
                lun,
            } => display_atapi(display_only, primary, slave, lun),
            MessagingNode::Scsi { target, lun } => {
                format!("Scsi({},{})", target, lun)
            }
            MessagingNode::FiberChannel { wwn, lun } => {
                format!("Fibre({},{})", wwn, lun)
            }
            MessagingNode::FiberChannelEx { wwn, lun } => {
                format!("FibreEx({},{})", wwn, lun)
            }
            MessagingNode::Sata {
                hba_port,
                port_multiplier_port,
                lun,
            } => {
                format!("Sata({},{},{})", hba_port, port_multiplier_port, lun)
            }
            MessagingNode::Usb {
                parent_port_number,
                usb_interface,
            } => {
                format!("Usb({},{})", parent_port_number, usb_interface)
            }
            MessagingNode::Lun(lun) => format!("Lun({})", lun),
            MessagingNode::UsbClass {
                vendor_id,
                product_id,
                device_class,
                device_subclass,
                device_protocol,
            } => display_usb_class(
                vendor_id,
                product_id,
                device_class,
                device_subclass,
                device_protocol,
            ),
            MessagingNode::MacAddr { mac_addr, if_type } => {
                format!("MAC({},{})", mac_addr, if_type)
            }
            MessagingNode::IpV4 {
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                is_static,
                gw,
                mask,
            } => display_ip_v4(
                display_only,
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                is_static,
                gw,
                mask,
            ),
            MessagingNode::IpV6 {
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                origin,
                prefix_len,
                gw,
            } => display_ipv6(
                display_only,
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                origin,
                prefix_len,
                gw,
            ),
            MessagingNode::Vlan { vlan_id } => format!("Vlan({})", vlan_id),
            MessagingNode::IScsi {
                protocol: _,
                options: _,
                lun,
                group_tag,
                target,
            } => {
                if display_only {
                    format!("iSCSI({})", target)
                } else {
                    format!(
                        "iSCSI({},{},{})",
                        target,
                        group_tag,
                        hex::encode(lun.to_be_bytes())
                    )
                }
            }
            MessagingNode::Sd { slot } => format!("Sd({})", slot),
            MessagingNode::EMMC { slot } => format!("EMMC({})", slot),
            MessagingNode::Nvme {
                namespace_id,
                namespace_uuid,
            } => format!("Nvme({},{})", namespace_id, namespace_uuid),
            MessagingNode::Unknown(node) => format!(
                "{}({},{})",
                self.get_generic_name(),
                node.node_sub_type,
                node.data.as_ref().map_or("null".to_string(), hex::encode)
            ),
            MessagingNode::UsbWwid {
                interface_number,
                vendor_id,
                product_id,
                serial: _, //TODO: decode serial
            } => format!(
                "UsbWwid({},{},{},WWID)",
                vendor_id, product_id, interface_number,
            ),
            MessagingNode::I2O { tid } => format!("I2O({})", tid),
            MessagingNode::Uri { uri } => {
                if uri.is_empty() {
                    "Uri()".to_string()
                } else {
                    format!("Uri({})", uri)
                }
            }
        }
    }

    fn get_data(&self) -> Option<Vec<u8>> {
        match self {
            MessagingNode::Atapi {
                primary,
                slave,
                lun,
            } => {
                let mut data = Vec::new();
                data.push(if *primary { 0 } else { 1 });
                data.push(if *slave { 1 } else { 0 });
                data.extend_from_slice(&lun.to_le_bytes());
                Some(data)
            }
            MessagingNode::Scsi { target, lun } => {
                let mut data = Vec::new();
                data.extend_from_slice(&target.to_le_bytes());
                data.extend_from_slice(&lun.to_le_bytes());
                Some(data)
            }
            MessagingNode::FiberChannel { wwn, lun } => {
                let mut data = Vec::new();
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&wwn.to_le_bytes());
                data.extend_from_slice(&lun.to_le_bytes());
                Some(data)
            }
            MessagingNode::FiberChannelEx { wwn, lun } => {
                let mut data = Vec::new();
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&wwn.to_le_bytes());
                data.extend_from_slice(&lun.to_le_bytes());
                Some(data)
            }
            MessagingNode::Sata {
                hba_port,
                port_multiplier_port,
                lun,
            } => {
                let mut data = Vec::new();
                data.extend_from_slice(&hba_port.to_le_bytes());
                data.extend_from_slice(&port_multiplier_port.to_le_bytes());
                data.extend_from_slice(&lun.to_le_bytes());
                Some(data)
            }
            MessagingNode::Usb {
                parent_port_number,
                usb_interface,
            } => {
                let mut data = Vec::new();
                data.push(*parent_port_number);
                data.push(*usb_interface);
                Some(data)
            }
            MessagingNode::Lun(lun) => Some(vec![*lun]),
            MessagingNode::UsbClass {
                vendor_id,
                product_id,
                device_class,
                device_subclass,
                device_protocol,
            } => {
                let mut data = Vec::new();
                data.extend_from_slice(&vendor_id.to_le_bytes());
                data.extend_from_slice(&product_id.to_le_bytes());
                data.push(*device_class);
                data.push(*device_subclass);
                data.push(*device_protocol);
                Some(data)
            }
            MessagingNode::MacAddr { mac_addr, if_type } => {
                let mut data = vec![0; 32];
                if mac_addr.is_v6() {
                    data[0..6].copy_from_slice(&mac_addr.as_bytes());
                } else {
                    data[0..8].copy_from_slice(&mac_addr.as_bytes());
                }
                data.push(*if_type);
                Some(data)
            }
            MessagingNode::IpV4 {
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                is_static,
                gw,
                mask,
            } => {
                let mut data = Vec::new();
                data.extend_from_slice(&local_ip.octets());
                data.extend_from_slice(&remote_ip.octets());
                data.extend_from_slice(&local_port.to_le_bytes());
                data.extend_from_slice(&remote_port.to_le_bytes());
                data.extend_from_slice(&protocol.to_le_bytes());
                data.push(if *is_static { 1 } else { 0 });
                data.extend_from_slice(&gw.octets());
                data.extend_from_slice(&mask.octets());
                Some(data)
            }
            MessagingNode::IpV6 {
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                origin,
                prefix_len,
                gw,
            } => {
                let mut data = Vec::new();
                data.extend_from_slice(&local_ip.octets());
                data.extend_from_slice(&remote_ip.octets());
                data.extend_from_slice(&local_port.to_le_bytes());
                data.extend_from_slice(&remote_port.to_le_bytes());
                data.extend_from_slice(&protocol.to_le_bytes());
                data.push(*origin);
                data.push(*prefix_len);
                data.extend_from_slice(&gw.octets());
                Some(data)
            }
            MessagingNode::Vlan { vlan_id } => Some(vlan_id.to_le_bytes().to_vec()),
            MessagingNode::IScsi {
                protocol,
                options,
                lun,
                group_tag,
                target,
            } => {
                let mut data = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut data);
                cursor.write_u16::<LittleEndian>(*protocol).ok()?;
                cursor.write_u16::<LittleEndian>(*options).ok()?;
                cursor.write_u64::<LittleEndian>(*lun).ok()?;
                cursor.write_u16::<LittleEndian>(*group_tag).ok()?;
                cursor.write_as_null_terminated_ascii(target).ok()?;

                Some(data)
            }
            MessagingNode::Sd { slot } => Some(vec![*slot]),
            MessagingNode::EMMC { slot } => Some(vec![*slot]),
            MessagingNode::Nvme {
                namespace_id,
                namespace_uuid,
            } => {
                let mut data = Vec::new();
                data.extend_from_slice(&namespace_id.to_le_bytes());
                data.extend_from_slice(&namespace_uuid.to_le_bytes());
                Some(data)
            }
            MessagingNode::I2O { tid } => Some(tid.to_le_bytes().to_vec()),
            MessagingNode::Uri { uri } => {
                if uri.is_empty() {
                    None
                } else {
                    Some(uri.as_bytes().to_vec())
                }
            }
            MessagingNode::Unknown(node) => node.data.clone(),
            MessagingNode::UsbWwid {
                interface_number,
                vendor_id,
                product_id,
                serial,
            } => {
                let mut data = Vec::new();
                data.extend_from_slice(&interface_number.to_le_bytes());
                data.extend_from_slice(&vendor_id.to_le_bytes());
                data.extend_from_slice(&product_id.to_le_bytes());
                data.extend_from_slice(serial);
                Some(data)
            }
        }
    }
}

fn ata_controller_display(primary: bool) -> &'static str {
    if primary {
        "Primary"
    } else {
        "Secondary"
    }
}

fn ata_drive_display(slave: bool) -> &'static str {
    if slave {
        "Slave"
    } else {
        "Master"
    }
}

fn display_atapi(display_only: bool, primary: &bool, slave: &bool, lun: &u16) -> String {
    if display_only {
        format!("Ata({})", lun)
    } else {
        format!(
            "Ata({},{},{})",
            ata_controller_display(*primary),
            ata_drive_display(*slave),
            lun
        )
    }
}

fn usb_class_to_string(class: u8) -> &'static str {
    match class {
        1 => "UsbAudio",
        2 => "UsbCDCControl",
        3 => "UsbHID",
        6 => "UsbImage",
        7 => "UsbPrinter",
        8 => "UsbMassStorage",
        9 => "UsbHub",
        10 => "UsbCDCData",
        11 => "UsbSmartCard",
        14 => "UsbVideo",
        220 => "UsbDiagnostic",
        224 => "UsbWireless",
        _ => "UsbClass",
    }
}

fn usb_class254_subclass_to_string(sub_class: u8) -> &'static str {
    match sub_class {
        1 => "UsbDeviceFirmwareUpdate",
        2 => "UsbIrdaBridge",
        3 => "UsbTestAndMeasurement",
        _ => "",
    }
}

fn display_usb_class(
    vendor_id: &u16,
    product_id: &u16,
    device_class: &u8,
    device_subclass: &u8,
    device_protocol: &u8,
) -> String {
    match device_class {
        254 => match device_subclass {
            1 | 2 | 3 => {
                let name = usb_class254_subclass_to_string(*device_subclass);
                format!("{}({},{},{})", name, vendor_id, product_id, device_protocol)
            }
            _ => format!(
                "UsbClass({},{},{},{},{})",
                vendor_id, product_id, device_class, device_subclass, device_protocol
            ),
        },
        1 | 2 | 3 | 6 | 7 | 8 | 9 | 10 | 11 | 14 | 220 | 224 => {
            let class = usb_class_to_string(*device_class);
            format!(
                "{}({},{},{},{})",
                class, vendor_id, product_id, device_subclass, device_protocol
            )
        }
        _ => format!(
            "UsbClass({},{},{},{},{})",
            vendor_id, product_id, device_class, device_subclass, device_protocol
        ),
    }
}

fn display_ipv6(
    display_only: bool,
    local_ip: &Ipv6Addr,
    remote_ip: &Ipv6Addr,
    local_port: &u16,
    remote_port: &u16,
    protocol: &u16,
    origin: &u8,
    prefix_len: &u8,
    gw: &Ipv6Addr,
) -> String {
    if display_only {
        format!("IPv6({})", remote_ip)
    } else {
        let protocol = match protocol {
            0x06 => "TCP".to_string(),
            0x11 => "UDP".to_string(),
            _ => protocol.to_string(),
        };
        let origin = match origin {
            0 => "Static".to_string(),
            1 => "StatelessAutoConfigure".to_string(),
            2 => "StatefulAutoConfigure".to_string(),
            _ => origin.to_string(),
        };
        format!(
            "IPv6({}:{},{},{},{}:{},{},{})",
            remote_ip, remote_port, protocol, origin, local_ip, local_port, gw, prefix_len
        )
    }
}

fn display_ip_v4(
    display_only: bool,
    local_ip: &Ipv4Addr,
    remote_ip: &Ipv4Addr,
    local_port: &u16,
    remote_port: &u16,
    protocol: &u16,
    is_static: &bool,
    gw: &Ipv4Addr,
    mask: &Ipv4Addr,
) -> String {
    if display_only {
        format!("IPv4({})", remote_ip)
    } else {
        let protocol = match protocol {
            0x06 => "TCP".to_string(),
            0x11 => "UDP".to_string(),
            _ => protocol.to_string(),
        };
        let is_static = if *is_static { "Static" } else { "DHCP" };
        format!(
            "IPv4({}:{},{},{},{}:{},{},{})",
            remote_ip, remote_port, protocol, is_static, local_ip, local_port, gw, mask
        )
    }
}

// FIXME: there is a note somewhere in the UEFI spec saying that MAC must be exactly
// 6 byte depends on the interface type but I cannot find this place again :)
fn parse_mac(padded_mac: [u8; 32]) -> Result<MacAddr> {
    // Check if the array is a 6-byte MAC followed by all zeros
    if padded_mac[6..].iter().all(|&b| b == 0) {
        let mac_bytes: [u8; 6] = padded_mac[0..6].try_into()?;
        MacAddr::try_from(mac_bytes).context("invalid 6-byte mac address")
    }
    // Check if it's an 8-byte EUI-64 followed by all zeros
    else if padded_mac[8..].iter().all(|&b| b == 0) {
        let mac_bytes: [u8; 8] = padded_mac[0..8].try_into()?;
        MacAddr::try_from(mac_bytes).context("invalid 8-byte mac address")
    }
    // Neither case matches
    else {
        Err(anyhow!(
            "Unexpected number of padding 0s parsing MAC address"
        ))
    }
}

impl TryFrom<&Node> for MessagingNode {
    type Error = anyhow::Error;

    fn try_from(value: &Node) -> Result<Self, Self::Error> {
        let subtype = DevicePathSubTypeMessaging::from_primitive(value.node_sub_type);
        println!("SUBTYPE: {:#?}, len={}", subtype, value.node_length);
        subtype.validate_length(value.node_length)?;

        // For Unknown and Uri types, handle specially as they can have node_length == 4
        match subtype {
            DevicePathSubTypeMessaging::Unknown(_) => {
                // Unknown nodes can have no data if node_length == 4
                return Ok(MessagingNode::Unknown(value.clone()));
            }
            DevicePathSubTypeMessaging::Uri => {
                // Uri can be empty (length == 4) per RFC 3986 / UEFI spec
                if value.node_length == 4 {
                    return Ok(MessagingNode::Uri { uri: String::new() });
                }
                // URI is stored as ASCII string (not null-terminated, length-delimited)
                let data = value.data.as_ref().ok_or_else(|| {
                    anyhow!("Node data is None but node_length is {}", value.node_length)
                })?;
                // RFC 3986 URIs are ASCII (with percent-encoding for non-ASCII)
                let uri = String::from_utf8(data.clone())
                    .context("Invalid ASCII/UTF-8 in URI - URIs must be RFC 3986 compliant")?;
                return Ok(MessagingNode::Uri { uri });
            }
            _ => {
                // All other known node types require data
                let data = value.data.as_ref().ok_or_else(|| {
                    anyhow!("Node data is None but node_length is {}", value.node_length)
                })?;
                let mut cursor = std::io::Cursor::new(data);

                parse_known_messaging_node(&mut cursor, subtype)
            }
        }
    }
}

fn parse_known_messaging_node(
    cursor: &mut std::io::Cursor<&Vec<u8>>,
    subtype: DevicePathSubTypeMessaging,
) -> Result<MessagingNode> {
    match subtype {
        DevicePathSubTypeMessaging::Atapi => {
            let primary = cursor.read_u8()? == 0;
            let slave = cursor.read_u8()? == 1;
            let lun = cursor.read_u16::<LittleEndian>()?;
            Ok(MessagingNode::Atapi {
                primary,
                slave,
                lun,
            })
        }
        DevicePathSubTypeMessaging::Scsi => {
            let target = cursor.read_u16::<LittleEndian>()?;
            let lun = cursor.read_u16::<LittleEndian>()?;
            Ok(MessagingNode::Scsi { target, lun })
        }
        DevicePathSubTypeMessaging::FiberChannel => {
            let _reserved = cursor.read_u32::<LittleEndian>()?;
            // those are not just u64
            let wwn = cursor.read_u64::<LittleEndian>()?;
            let lun = cursor.read_u64::<LittleEndian>()?;
            Ok(MessagingNode::FiberChannel { wwn, lun })
        }
        DevicePathSubTypeMessaging::FiberChannelEx => {
            let _reserved = cursor.read_u32::<LittleEndian>()?;
            // those are not just u64
            let wwn = cursor.read_u64::<LittleEndian>()?;
            let lun = cursor.read_u64::<LittleEndian>()?;
            // let boot_lun = cursor.read_u64::<LittleEndian>?;
            Ok(MessagingNode::FiberChannelEx { wwn, lun })
        }
        DevicePathSubTypeMessaging::Sata => {
            let hba_port = cursor.read_u16::<LittleEndian>()?;
            let port_multiplier_port = cursor.read_u16::<LittleEndian>()?;
            let lun = cursor.read_u16::<LittleEndian>()?;
            Ok(MessagingNode::Sata {
                hba_port,
                port_multiplier_port,
                lun,
            })
        }
        DevicePathSubTypeMessaging::Usb => {
            let parent_port_number = cursor.read_u8()?;
            let usb_interface = cursor.read_u8()?;
            Ok(MessagingNode::Usb {
                parent_port_number,
                usb_interface,
            })
        }
        DevicePathSubTypeMessaging::Lun => {
            let lun = cursor.read_u8()?;
            Ok(MessagingNode::Lun(lun))
        }
        DevicePathSubTypeMessaging::UsbClass => {
            let vendor_id = cursor.read_u16::<LittleEndian>()?;
            let product_id = cursor.read_u16::<LittleEndian>()?;
            let device_class = cursor.read_u8()?;
            let device_subclass = cursor.read_u8()?;
            let device_protocol = cursor.read_u8()?;
            Ok(MessagingNode::UsbClass {
                vendor_id,
                product_id,
                device_class,
                device_subclass,
                device_protocol,
            })
        }
        DevicePathSubTypeMessaging::UsbWwid => {
            let interface_number = cursor.read_u16::<LittleEndian>()?;
            let vendor_id = cursor.read_u16::<LittleEndian>()?;
            let product_id = cursor.read_u16::<LittleEndian>()?;
            let mut serial = Vec::new();
            _ = cursor.read_to_end(&mut serial)?;
            Ok(MessagingNode::UsbWwid {
                interface_number,
                vendor_id,
                product_id,
                serial,
            })
        }
        DevicePathSubTypeMessaging::MacAddr => {
            let mut mac_addr = [0; 32];
            cursor.read_exact(&mut mac_addr)?;
            let if_type = cursor.read_u8()?;
            let mac_addr = parse_mac(mac_addr)?;
            Ok(MessagingNode::MacAddr { mac_addr, if_type })
        }
        DevicePathSubTypeMessaging::IpV4 => {
            let local_ip = Ipv4Addr::from(cursor.read_u32::<LittleEndian>()?);
            let remote_ip = Ipv4Addr::from(cursor.read_u32::<LittleEndian>()?);
            let local_port = cursor.read_u16::<LittleEndian>()?;
            let remote_port = cursor.read_u16::<LittleEndian>()?;
            let protocol = cursor.read_u16::<LittleEndian>()?;
            let is_static = cursor.read_u8()? == 1;
            let gw = Ipv4Addr::from(cursor.read_u32::<LittleEndian>()?);
            let mask = Ipv4Addr::from(cursor.read_u32::<LittleEndian>()?);
            Ok(MessagingNode::IpV4 {
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                is_static,
                gw,
                mask,
            })
        }
        DevicePathSubTypeMessaging::IpV6 => {
            let local_ip = Ipv6Addr::from(cursor.read_u128::<LittleEndian>()?);
            let remote_ip = Ipv6Addr::from(cursor.read_u128::<LittleEndian>()?);
            let local_port = cursor.read_u16::<LittleEndian>()?;
            let remote_port = cursor.read_u16::<LittleEndian>()?;
            let protocol = cursor.read_u16::<LittleEndian>()?;
            let origin = cursor.read_u8()?;
            let prefix_len = cursor.read_u8()?;
            let gw = Ipv6Addr::from(cursor.read_u128::<LittleEndian>()?);
            Ok(MessagingNode::IpV6 {
                local_ip,
                remote_ip,
                local_port,
                remote_port,
                protocol,
                origin,
                prefix_len,
                gw,
            })
        }
        DevicePathSubTypeMessaging::IScsi => {
            let protocol = cursor.read_u16::<LittleEndian>()?;
            let options = cursor.read_u16::<LittleEndian>()?;
            let lun = cursor.read_u64::<LittleEndian>()?;
            let group_tag = cursor.read_u16::<LittleEndian>()?;
            // FIXME: it is unclear from the spec whether it is ucs16 or ascii
            let target = cursor.read_null_terminated_ascii_to_string()?;
            Ok(MessagingNode::IScsi {
                protocol,
                options,
                lun,
                group_tag,
                target,
            })
        }
        DevicePathSubTypeMessaging::Vlan => {
            let vlan_id = cursor.read_u16::<LittleEndian>()?;
            Ok(MessagingNode::Vlan { vlan_id })
        }
        DevicePathSubTypeMessaging::I2O => {
            let i2o_path_id = cursor.read_u32::<LittleEndian>()?;
            Ok(MessagingNode::I2O { tid: i2o_path_id })
        }
        DevicePathSubTypeMessaging::Nvme => {
            let namespace_id = cursor.read_u32::<LittleEndian>()?;
            let namespace_uuid = cursor.read_u64::<LittleEndian>()?;
            Ok(MessagingNode::Nvme {
                namespace_id,
                namespace_uuid,
            })
        }
        DevicePathSubTypeMessaging::Sd => {
            let slot = cursor.read_u8()?;
            Ok(MessagingNode::Sd { slot })
        }
        DevicePathSubTypeMessaging::EMMC => {
            let slot = cursor.read_u8()?;
            Ok(MessagingNode::EMMC { slot })
        }
        DevicePathSubTypeMessaging::Uri => {
            unreachable!(
                "Uri type should be handled in try_from before calling parse_known_messaging_node"
            )
        }
        DevicePathSubTypeMessaging::Unknown(_) => {
            unreachable!("Unknown type should be handled in try_from before calling parse_known_messaging_node")
        }
    }
}
