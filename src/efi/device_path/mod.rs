pub mod acpi;
pub mod hardware;
pub mod media;
pub mod messaging;
#[cfg(test)]
mod tests;
mod traits;

use std::io::{Cursor, Read};

use acpi::AcpiNode;
use byteorder::{LittleEndian, ReadBytesExt};
use hardware::HardwareNode;
use media::MediaNode;
use num_enum::TryFromPrimitive;
use strum::Display;
use traits::{DevicePathDisplay, NodeExpectedLength, NodeTypeValidator};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Node {
    node_type: u8,
    node_sub_type: u8,
    node_length: u16,
    data: Option<Vec<u8>>,
}

impl Node {
    fn is_end(&self) -> bool {
        self.node_type == DevicePathType::End as u8
            && self.node_sub_type == DevicePathSubTypeEnd::EndEntire as u8
    }
}

#[derive(Debug, TryFromPrimitive, Display)]
#[repr(u8)]
enum DevicePathType {
    Hardware = 0x1,
    ACPI = 0x2,
    Messaging = 0x3,
    Media = 0x4,
    BIOS = 0x5,
    End = 0x7f,
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
enum DevicePathSubTypeEnd {
    EndInstance = 0x1,
    EndEntire = 0xff,
}

impl NodeTypeValidator for DevicePathSubTypeEnd {
    fn expected_length(&self) -> NodeExpectedLength {
        match self {
            DevicePathSubTypeEnd::EndInstance => NodeExpectedLength::Exact(4),
            DevicePathSubTypeEnd::EndEntire => NodeExpectedLength::Exact(4),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum PathNode {
    Acpi(AcpiNode),
    Hardware(HardwareNode),
    Media(MediaNode),
    // Messaging(MessagingNode),
    EndInstance,
    EndEntire,
    Unknown(Node),
}

impl PathNode {
    fn is_end(&self) -> bool {
        match self {
            PathNode::Unknown(node) => node.is_end(),
            _ => false,
        }
    }
}

impl TryFrom<Node> for PathNode {
    type Error = anyhow::Error;

    fn try_from(value: Node) -> std::result::Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&Node> for PathNode {
    type Error = anyhow::Error;

    fn try_from(value: &Node) -> std::result::Result<Self, Self::Error> {
        match DevicePathType::try_from(value.node_type) {
            Ok(DevicePathType::End) => {
                let subtype = DevicePathSubTypeEnd::try_from(value.node_sub_type)?;
                subtype.validate_length(value.node_length)?;
                match subtype {
                    DevicePathSubTypeEnd::EndInstance => Ok(PathNode::EndInstance),
                    DevicePathSubTypeEnd::EndEntire => Ok(PathNode::EndEntire),
                }
            }
            Ok(DevicePathType::ACPI) => Ok(PathNode::Acpi(AcpiNode::try_from(value)?)),
            Ok(DevicePathType::Hardware) => Ok(PathNode::Hardware(HardwareNode::try_from(value)?)),
            Ok(DevicePathType::Media) => Ok(PathNode::Media(MediaNode::try_from(value)?)),
            _ => Ok(PathNode::Unknown(value.clone())),
        }
    }
}

impl DevicePathDisplay for PathNode {
    fn display(&self, display_only: bool) -> String {
        match self {
            PathNode::Acpi(acpi) => acpi.display(display_only),
            PathNode::Hardware(hardware) => hardware.display(display_only),
            PathNode::Media(media) => media.display(display_only),
            PathNode::EndInstance => "".to_string(),
            PathNode::EndEntire => "".to_string(),
            PathNode::Unknown(node) => format!(
                "Path({},{},{})",
                node.node_type,
                node.node_sub_type,
                hex::encode(node.data.as_ref().unwrap())
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DevicePath {
    pub nodes: Vec<PathNode>,
}

impl DevicePath {
    pub fn display(&self, display_only: bool) -> String {
        self.nodes
            .iter()
            .map(|node| node.display(display_only))
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn new() -> Self {
        DevicePath { nodes: Vec::new() }
    }

    pub fn acpi_acpi(mut self, pnp_id: u16, uid: u32) -> Self {
        let hid = (pnp_id as u32) << 16 | 0x41D0;
        let node = PathNode::Acpi(AcpiNode::Acpi(hid, uid));
        self.nodes.push(node);
        self
    }

    // pub fn msg_mac_addr(mut self, mac_addr: MacAddr, if_type: u8) -> Self {
    //     let node = DevicePathNode::MessagingMacAddr { mac_addr, if_type };
    //     self.nodes.push(node);
    //     self
    // }

    pub fn end_instance(mut self) -> Self {
        self.nodes.push(PathNode::EndInstance);
        self
    }

    pub fn end(mut self) -> Self {
        self.nodes.push(PathNode::EndEntire);
        self
    }

    pub fn hw_pci(mut self, function: u8, device: u8) -> Self {
        self.nodes
            .push(PathNode::Hardware(HardwareNode::Pci { function, device }));
        self
    }

    // pub fn msg_ipv4(
    //     mut self,
    //     local_ip: Ipv4Addr,
    //     remote_ip: Ipv4Addr,
    //     local_port: u16,
    //     remote_port: u16,
    //     is_static: bool,
    //     protocol: u16,
    //     gw: Ipv4Addr,
    //     net_mask: Ipv4Addr,
    // ) -> Self {
    //     self.nodes.push(DevicePathNode::MessagingIpV4 {
    //         local_ip,
    //         remote_ip,
    //         local_port,
    //         remote_port,
    //         protocol,
    //         is_static,
    //         gw,
    //         mask: net_mask,
    //     });
    //     self
    // }

    // pub fn msg_iSCSI(
    //     mut self,
    //     options: u16,
    //     target_port_gropup: u16,
    //     lun: u64,
    //     target: String,
    // ) -> Self {
    //     self.nodes.push(DevicePathNode::Messaging_iSCSI {
    //         protocol: 0, // 0 = TCP, 1+ reserved
    //         options,
    //         lun,
    //         group_tag: target_port_gropup,
    //         target,
    //     });
    //     self
    // }

    // pub fn media_hdd(
    //     mut self,
    //     partition_number: u32,
    //     partition_start: u64,
    //     partition_size: u64,
    //     signature: [u8; 16],
    //     partition_format: u8,
    //     signature_type: u8,
    // ) -> Self {
    //     self.nodes.push(DevicePathNode::MediaHardDrive {
    //         partition_number,
    //         partition_start,
    //         partition_size,
    //         signature,
    //         partition_format,
    //         signature_type,
    //     });
    //     self
    // }
}

impl TryFrom<&[u8]> for DevicePath {
    type Error = anyhow::Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut nodes = Vec::new();
        let mut cursor = Cursor::new(data);
        loop {
            let node_type = cursor.read_u8()?;
            let node_sub_type = cursor.read_u8()?;
            let node_length = cursor.read_u16::<LittleEndian>()?;
            let node_data = if node_length > 4 {
                let mut data = vec![0; node_length as usize - 4];
                cursor.read_exact(&mut data)?;
                Some(data)
            } else {
                None
            };
            let node = Node {
                node_type,
                node_sub_type,
                node_length,
                data: node_data,
            };
            if node.is_end() {
                break;
            }
            let node = PathNode::try_from(node)?;
            nodes.push(node);
        }

        Ok(DevicePath { nodes })
    }
}

fn eisa_id_to_acpi_device_path_string(hid: u32, uid: u32) -> String {
    let acpi_type = match hid >> 16 {
        0x0A03 => "PciRoot",
        0x0A08 => "PcieRoot",
        0x0604 => "Floppy",
        0x0301 => "Keyboard",
        0x0501 => "Serial",
        0x0401 => "ParallelPort",
        _ => "Acpi",
    };
    if acpi_type == "Acpi" {
        return format!("Acpi({:#X},{:#X})", hid, uid);
    }
    format!("{}({:#X})", acpi_type, uid)
}
