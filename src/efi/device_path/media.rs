use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use strum::Display;

use super::{
    traits::{DevicePathDisplay, DevicePathTypeReader, NodeExpectedLength, NodeTypeValidator},
    Node,
};

#[derive(Debug, TryFromPrimitive, Display)]
#[repr(u8)]
enum NodeSubTypeMedia {
    HardDrive = 0x1,
    CdromElTorito = 0x2,
    Vendor = 0x3,
    FilePath = 0x4,
    MediaProtocol = 0x5,
    PiwgFvFile = 0x6,
    PiwgFv = 0x7,
    RelativeOffsetRange = 0x8,
    RamDisk = 0x9,
}

impl NodeTypeValidator for NodeSubTypeMedia {
    fn expected_length(&self) -> NodeExpectedLength {
        match self {
            NodeSubTypeMedia::HardDrive => NodeExpectedLength::Exact(42),
            NodeSubTypeMedia::CdromElTorito => NodeExpectedLength::Exact(24),
            NodeSubTypeMedia::Vendor => NodeExpectedLength::Min(20),
            NodeSubTypeMedia::FilePath => NodeExpectedLength::Min(4),
            NodeSubTypeMedia::MediaProtocol => NodeExpectedLength::Exact(20),
            NodeSubTypeMedia::PiwgFvFile => NodeExpectedLength::Exact(20),
            NodeSubTypeMedia::PiwgFv => NodeExpectedLength::Exact(20),
            NodeSubTypeMedia::RelativeOffsetRange => NodeExpectedLength::Exact(24),
            NodeSubTypeMedia::RamDisk => NodeExpectedLength::Exact(38),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PartitionType {
    Mbr,
    Gpt,
    Unknown(u8),
}

impl From<u8> for PartitionType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => PartitionType::Mbr,
            0x02 => PartitionType::Gpt,
            _ => PartitionType::Unknown(value),
        }
    }
}

impl std::fmt::Display for PartitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionType::Mbr => write!(f, "MBR"),
            PartitionType::Gpt => write!(f, "GPT"),
            PartitionType::Unknown(value) => write!(f, "{:02x}", value),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PartitionSignature {
    None,
    Mbr(u16),
    Guid(uuid::Uuid),
}

impl std::fmt::Display for PartitionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionSignature::None => write!(f, ""),
            PartitionSignature::Mbr(value) => write!(f, "{:04x}", value),
            PartitionSignature::Guid(value) => write!(
                f,
                "{}",
                value
                    .as_hyphenated()
                    .encode_lower(&mut uuid::Uuid::encode_buffer())
            ),
        }
    }
}

impl PartitionSignature {
    fn new(kind: u8, value: &[u8; 16]) -> Self {
        if kind == 1 {
            PartitionSignature::Mbr(u16::from_le_bytes([value[0], value[1]]))
        } else {
            PartitionSignature::Guid(uuid::Uuid::from_slice(value).unwrap())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum MediaNode {
    HardDrive {
        partition_number: u32,
        partition_start: u64,
        partition_size: u64,
        signature: PartitionSignature,
        partition_format: PartitionType,
    },
    CdRom {
        boot_entry: u32,
        partition_start: u64,
        partition_size: u64,
    },
    Vendor {
        guid: uuid::Uuid,
        vendor_data: Vec<u8>,
    },
    FilePath(String),
    PiwgFvFile(uuid::Uuid), // TODO: implement
    PiwgFv(uuid::Uuid),
    Unknown(Node),
}

impl DevicePathDisplay for MediaNode {
    fn display(&self, display: bool) -> String {
        match self {
            MediaNode::HardDrive {
                partition_number,
                partition_start,
                partition_size,
                signature,
                partition_format,
            } => {
                if display || *partition_number == 0 {
                    format!(
                        "HD({},{},{})",
                        partition_number, partition_format, signature
                    )
                } else {
                    format!(
                        "HD({},{},{},{},{})",
                        partition_number,
                        partition_format,
                        signature,
                        partition_start,
                        partition_size
                    )
                }
            }
            MediaNode::CdRom {
                boot_entry,
                partition_start,
                partition_size,
            } => {
                if display {
                    format!(
                        "CdRom({},{},{})",
                        boot_entry, partition_start, partition_size
                    )
                } else {
                    "CdRom".to_string()
                }
            }
            MediaNode::Vendor { guid, vendor_data } => {
                if display {
                    format!("Vendor({},{:?})", guid, hex::encode_upper(vendor_data))
                } else {
                    "Vendor".to_string()
                }
            }
            MediaNode::FilePath(path) => path.clone(),
            MediaNode::PiwgFvFile(guid) => format!("FvFile({})", guid),
            MediaNode::PiwgFv(guid) => format!("Fv({})", guid),
            MediaNode::Unknown(node) => format!(
                "MediaPath({},{})",
                node.node_sub_type,
                hex::encode(node.data.as_ref().unwrap())
            ),
        }
    }
}

impl TryFrom<&Node> for MediaNode {
    type Error = anyhow::Error;

    fn try_from(node: &Node) -> std::result::Result<Self, Self::Error> {
        match NodeSubTypeMedia::try_from(node.node_sub_type) {
            Ok(subtype) => {
                subtype.validate_length(node.node_length)?;
                let mut cursor = Cursor::new(node.data.as_ref().unwrap());

                match subtype {
                    NodeSubTypeMedia::HardDrive => {
                        let partition_number = cursor.read_u32::<LittleEndian>()?;
                        let partition_start = cursor.read_u64::<LittleEndian>()?;
                        let partition_size = cursor.read_u64::<LittleEndian>()?;
                        let mut signature = [0; 16];
                        cursor.read_exact(&mut signature)?;
                        let partition_format = cursor.read_u8()?;
                        let signature_type = cursor.read_u8()?;
                        Ok(MediaNode::HardDrive {
                            partition_number,
                            partition_start,
                            partition_size,
                            signature: PartitionSignature::new(signature_type, &signature),
                            partition_format: PartitionType::from(partition_format),
                        })
                    }
                    NodeSubTypeMedia::CdromElTorito => Ok(MediaNode::CdRom {
                        boot_entry: cursor.read_u32::<LittleEndian>()?,
                        partition_start: cursor.read_u64::<LittleEndian>()?,
                        partition_size: cursor.read_u64::<LittleEndian>()?,
                    }),
                    NodeSubTypeMedia::Vendor => {
                        let mut vendor_data = Vec::new();
                        let guid = cursor.read_guid()?;
                        let _data_size = cursor.read_to_end(&mut vendor_data)?;
                        Ok(MediaNode::Vendor { guid, vendor_data })
                    }
                    NodeSubTypeMedia::FilePath => {
                        let path = cursor.read_ucs16_null_terminated_to_string()?;
                        Ok(MediaNode::FilePath(path))
                    }
                    NodeSubTypeMedia::PiwgFvFile => Ok(MediaNode::PiwgFvFile(cursor.read_guid()?)),
                    NodeSubTypeMedia::PiwgFv => Ok(MediaNode::PiwgFv(cursor.read_guid()?)),
                    NodeSubTypeMedia::RelativeOffsetRange => todo!(),
                    NodeSubTypeMedia::RamDisk => todo!(),
                    NodeSubTypeMedia::MediaProtocol => todo!(),
                }
            }
            Err(_) => Ok(MediaNode::Unknown(node.clone())),
        }
    }
}
