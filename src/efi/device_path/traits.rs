use std::io::{Cursor, Read};

use anyhow::{anyhow, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};

pub enum NodeExpectedLength {
    Exact(u16),
    Min(u16),
}

pub(super) trait NodeTypeValidator {
    fn expected_length(&self) -> NodeExpectedLength;
    fn validate_length(&self, length: u16) -> Result<()>
    where
        Self: std::fmt::Debug,
    {
        match self.expected_length() {
            NodeExpectedLength::Exact(expected) => {
                if length != expected {
                    return Err(anyhow!(
                        "invalid length for acpi device path sub type {:?}: expected {}, got {}",
                        self,
                        expected,
                        length
                    ));
                }
            }
            NodeExpectedLength::Min(min) => {
                if length < min {
                    return Err(anyhow!(
                        "invalid length for acpi device path sub type {:?}: expected at least {}, got {}",
                        self,
                        min,
                        length
                    ));
                }
            }
        }
        Ok(())
    }
}

pub(super) trait DevicePathDisplay {
    fn display(&self, display: bool) -> String;
}

pub(super) trait DevicePathTypeReader {
    fn read_null_terminated_ascii_to_string(&mut self) -> Result<String>;
    fn read_ucs16_null_terminated_to_string(&mut self) -> Result<String>;
    fn read_guid(&mut self) -> Result<uuid::Uuid>;
}

impl DevicePathTypeReader for Cursor<&Vec<u8>> {
    fn read_null_terminated_ascii_to_string(&mut self) -> Result<String> {
        let mut chars = Vec::new();
        loop {
            match self.read_u8() {
                Ok(0) => break,
                Ok(c) => {
                    if !c.is_ascii() {
                        return Err(anyhow!("invalid ascii control character: {}", c));
                    }
                    chars.push(c)
                }
                Err(e) => return Err(anyhow!("error reading null terminated string: {}", e)),
            }
        }
        Ok(String::from_utf8(chars).context("error converting ascii string")?)
    }
    fn read_guid(&mut self) -> Result<uuid::Uuid> {
        let mut guid = [0u8; 16];
        self.read_exact(&mut guid)?;
        Ok(uuid::Uuid::from_bytes(guid))
    }
    fn read_ucs16_null_terminated_to_string(&mut self) -> Result<String> {
        let mut chars = Vec::new();
        loop {
            match self.read_u16::<LittleEndian>() {
                Ok(0) => break,
                Ok(c) => chars.push(c),
                Err(e) => return Err(anyhow!("error reading null terminated string: {}", e)),
            }
        }
        Ok(String::from_utf16_lossy(&chars))
    }
}

impl DevicePathTypeReader for Cursor<Vec<u8>> {
    fn read_null_terminated_ascii_to_string(&mut self) -> Result<String> {
        let mut chars = Vec::new();
        loop {
            match self.read_u8() {
                Ok(0) => break,
                Ok(c) => {
                    if !c.is_ascii() {
                        return Err(anyhow!("invalid ascii control character: {}", c));
                    }
                    chars.push(c)
                }
                Err(e) => return Err(anyhow!("error reading null terminated string: {}", e)),
            }
        }
        Ok(String::from_utf8(chars).context("error converting ascii string")?)
    }
    fn read_guid(&mut self) -> Result<uuid::Uuid> {
        let mut guid = [0u8; 16];
        self.read_exact(&mut guid)?;
        Ok(uuid::Uuid::from_bytes(guid))
    }
    fn read_ucs16_null_terminated_to_string(&mut self) -> Result<String> {
        let mut chars = Vec::new();
        loop {
            match self.read_u16::<LittleEndian>() {
                Ok(0) => break,
                Ok(c) => chars.push(c),
                Err(e) => return Err(anyhow!("error reading null terminated string: {}", e)),
            }
        }
        Ok(String::from_utf16_lossy(&chars))
    }
}
