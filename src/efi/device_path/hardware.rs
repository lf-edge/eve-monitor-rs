use std::io::{Cursor, Read};

use anyhow::Context;
use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;

use super::{
    traits::{DevicePathDisplay, NodeExpectedLength, NodeTypeValidator},
    Node,
};

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
enum DevicePathSubTypeHardware {
    Pci = 0x1,
    PCCARD = 0x2,
    MemoryMapped = 0x3,
    Vendor = 0x4,
    Controller = 0x5,
    BMC = 0x6,
}

impl NodeTypeValidator for DevicePathSubTypeHardware {
    fn expected_length(&self) -> NodeExpectedLength {
        match self {
            DevicePathSubTypeHardware::Pci => NodeExpectedLength::Exact(6),
            DevicePathSubTypeHardware::PCCARD => NodeExpectedLength::Exact(5),
            DevicePathSubTypeHardware::MemoryMapped => NodeExpectedLength::Exact(24),
            DevicePathSubTypeHardware::Vendor => NodeExpectedLength::Min(20),
            DevicePathSubTypeHardware::Controller => NodeExpectedLength::Exact(8),
            DevicePathSubTypeHardware::BMC => NodeExpectedLength::Exact(13),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum HardwareNode {
    Pci {
        function: u8,
        device: u8,
    },
    PCCARD {
        function: u8,
    },
    MemoryMapped {
        memory_type: u32,
        start_address: u64,
        end_address: u64,
    },
    Vendor {
        guid: uuid::Uuid,
        data: Vec<u8>,
    },
    Controller {
        controller: u32,
    },
    BMC {
        interface: u8,
        base_address: u64,
    },

    Unknown(Node),
}

impl DevicePathDisplay for HardwareNode {
    fn display(&self, _display_only: bool) -> String {
        match self {
            HardwareNode::Pci { function, device } => format!("Pci({:#X},{:#X})", device, function),
            HardwareNode::PCCARD { function } => format!("PcCard({:#X})", function),
            HardwareNode::MemoryMapped {
                memory_type,
                start_address,
                end_address,
            } => format!(
                "MemoryMapped({:#X},{:#X},{:#X})",
                memory_type, start_address, end_address
            ),
            HardwareNode::Vendor { guid, data } => {
                format!("VenHw({},{})", guid, hex::encode(data))
            }
            HardwareNode::Controller { controller } => format!("Ctrl({:#X})", controller),
            HardwareNode::BMC {
                interface,
                base_address,
            } => format!("BMC({},{:#X})", interface, base_address),
            HardwareNode::Unknown(node) => format!(
                "HardwarePath({},{})",
                node.node_sub_type,
                hex::encode(node.data.as_ref().unwrap())
            ),
        }
    }
}

impl TryFrom<&Node> for HardwareNode {
    type Error = anyhow::Error;

    fn try_from(node: &Node) -> std::result::Result<Self, Self::Error> {
        match DevicePathSubTypeHardware::try_from(node.node_sub_type) {
            Ok(subtype) => {
                subtype.validate_length(node.node_length)?;
                let mut cursor = Cursor::new(node.data.as_ref().unwrap());

                match subtype {
                    DevicePathSubTypeHardware::Pci => Ok(HardwareNode::Pci {
                        function: cursor.read_u8().context("error reading function")?,
                        device: cursor.read_u8().context("error reading function")?,
                    }),
                    DevicePathSubTypeHardware::PCCARD => Ok(HardwareNode::PCCARD {
                        function: cursor
                            .read_u8()
                            .context("error reading function for PCCARD")?,
                    }),
                    DevicePathSubTypeHardware::MemoryMapped => Ok(HardwareNode::MemoryMapped {
                        memory_type: cursor
                            .read_u32::<LittleEndian>()
                            .context("error reading memory type")?,
                        start_address: cursor
                            .read_u64::<LittleEndian>()
                            .context("error reading start address")?,
                        end_address: cursor
                            .read_u64::<LittleEndian>()
                            .context("error reading end address")?,
                    }),
                    DevicePathSubTypeHardware::Vendor => {
                        let mut uuid_buffer: Vec<u8> = Vec::with_capacity(16);
                        let mut vendor_data = Vec::new();
                        cursor.read_exact(&mut uuid_buffer)?;
                        let guid = uuid::Uuid::from_slice(&uuid_buffer)?;
                        let _data_size = cursor.read_to_end(&mut vendor_data)?;
                        Ok(HardwareNode::Vendor {
                            guid,
                            data: vendor_data,
                        })
                    }
                    DevicePathSubTypeHardware::Controller => Ok(HardwareNode::Controller {
                        controller: cursor.read_u32::<LittleEndian>()?,
                    }),
                    DevicePathSubTypeHardware::BMC => Ok(HardwareNode::BMC {
                        interface: cursor.read_u8()?,
                        base_address: cursor.read_u64::<LittleEndian>()?,
                    }),
                }
            }
            Err(_) => Ok(HardwareNode::Unknown(node.clone())),
        }
    }
}
