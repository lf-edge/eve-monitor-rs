// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use uuid::Uuid;

use super::tcg_tpmlog::{TcgTpmEvent, TcgTpmEventType};

#[derive(Debug)]
pub struct TcgEfiVariableEvent {
    pub vendor_guid: Uuid,
    pub unicode_name: String,
    pub variable_data: Vec<u8>,
}

// corresponds to EV_IPL event for PCR 8
pub struct TcgIPLEvent(String);

impl TcgIPLEvent {
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&TcgTpmEvent> for TcgIPLEvent {
    type Error = anyhow::Error;

    fn try_from(value: &TcgTpmEvent) -> std::result::Result<Self, Self::Error> {
        if value.event_type != TcgTpmEventType::IPL {
            return Err(anyhow::anyhow!(
                "Invalid event type for IPL event {}",
                value.event_type
            ));
        }

        let event_data = std::str::from_utf8(&value.event_data)
            .context("Error converting event data to utf-8 string")?;

        Ok(TcgIPLEvent(event_data.to_string()))
    }
}

pub struct TcgEfiActionEvent(String);

impl TcgEfiActionEvent {
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl TryFrom<TcgTpmEvent> for TcgEfiActionEvent {
    type Error = anyhow::Error;

    fn try_from(value: TcgTpmEvent) -> std::result::Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&TcgTpmEvent> for TcgEfiActionEvent {
    type Error = anyhow::Error;

    fn try_from(value: &TcgTpmEvent) -> Result<Self> {
        if value.event_type != TcgTpmEventType::EfiAction {
            return Err(anyhow::anyhow!(
                "Invalid event type for action event {}",
                value.event_type
            ));
        }

        let evetnt_value = std::str::from_utf8(&value.event_data)
            .context("Error converting event data to utf-8 string")?;

        Ok(TcgEfiActionEvent(evetnt_value.to_string()))
    }
}

impl TryFrom<TcgTpmEvent> for TcgEfiVariableEvent {
    type Error = anyhow::Error;

    fn try_from(value: TcgTpmEvent) -> std::result::Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&TcgTpmEvent> for TcgEfiVariableEvent {
    type Error = anyhow::Error;

    fn try_from(value: &TcgTpmEvent) -> std::result::Result<Self, Self::Error> {
        if !value.event_type.is_efi_boot_variable() {
            return Err(anyhow::anyhow!(
                "Invalid event type for EFI variable boot event {}",
                value.event_type
            ));
        }

        let event_data = TcgEfiVariableEvent::parse(&value.event_data)?;

        Ok(event_data)
    }
}

impl TcgEfiVariableEvent {
    fn parse(data: &[u8]) -> Result<Self> {
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
