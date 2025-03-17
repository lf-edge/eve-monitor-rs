use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;

use super::{
    eisa_id_to_acpi_device_path_string,
    traits::{DevicePathDisplay, DevicePathTypeReader, NodeExpectedLength, NodeTypeValidator},
    Node,
};

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
enum DevicePathSubTypeAcpi {
    Acpi = 0x1,
    ExpandedAcpi = 0x2,
    Adr = 0x3,
}

impl NodeTypeValidator for DevicePathSubTypeAcpi {
    fn expected_length(&self) -> NodeExpectedLength {
        match self {
            DevicePathSubTypeAcpi::Acpi => NodeExpectedLength::Exact(12),
            DevicePathSubTypeAcpi::ExpandedAcpi => NodeExpectedLength::Min(19),
            DevicePathSubTypeAcpi::Adr => NodeExpectedLength::Min(8),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum AcpiNode {
    Acpi(u32, u32),
    AcpiExpanded(u32, u32, u32, String, String, String),
    AcpiAdr(u32, Option<Vec<u32>>),
    Unknown(Node),
}

impl DevicePathDisplay for AcpiNode {
    fn display(&self, display_only: bool) -> String {
        match self {
            AcpiNode::Acpi(hid, uid) => eisa_id_to_acpi_device_path_string(*hid, *uid),
            AcpiNode::AcpiExpanded(hid, uid, cid, hid_str, uid_str, cid_str) => todo!(),
            AcpiNode::AcpiAdr(adr, additional_addr) => todo!(),
            AcpiNode::Unknown(node) => format!(
                "AcpiPath({},{})",
                node.node_sub_type,
                hex::encode(node.data.as_ref().unwrap())
            ),
        }
    }
}

impl TryFrom<&Node> for AcpiNode {
    type Error = anyhow::Error;

    fn try_from(node: &Node) -> std::result::Result<Self, Self::Error> {
        match DevicePathSubTypeAcpi::try_from(node.node_sub_type) {
            Ok(subtype) => {
                subtype.validate_length(node.node_length)?;
                let mut cursor = Cursor::new(node.data.as_ref().unwrap());

                match subtype {
                    DevicePathSubTypeAcpi::Acpi => {
                        let hid = cursor.read_u32::<LittleEndian>()?;
                        let uid = cursor.read_u32::<LittleEndian>()?;
                        Ok(AcpiNode::Acpi(hid, uid))
                    }
                    DevicePathSubTypeAcpi::ExpandedAcpi => {
                        let hid = cursor.read_u32::<LittleEndian>()?;
                        let uid = cursor.read_u32::<LittleEndian>()?;
                        let cid = cursor.read_u32::<LittleEndian>()?;
                        let hid_str = cursor.read_null_terminated_ascii_to_string()?;
                        let uid_str = cursor.read_null_terminated_ascii_to_string()?;
                        let cid_str = cursor.read_null_terminated_ascii_to_string()?;
                        Ok(AcpiNode::AcpiExpanded(
                            hid, uid, cid, hid_str, uid_str, cid_str,
                        ))
                    }
                    DevicePathSubTypeAcpi::Adr => {
                        let hid = cursor.read_u32::<LittleEndian>()?;
                        let count = cursor.read_u8()?;
                        let mut adrs = Vec::new();
                        for _ in 0..count {
                            adrs.push(cursor.read_u32::<LittleEndian>()?);
                        }
                        Ok(AcpiNode::AcpiAdr(hid, Some(adrs)))
                    }
                }
            }
            Err(_) => Ok(AcpiNode::Unknown(node.clone())),
        }
    }
}
