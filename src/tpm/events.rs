// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use uuid::Uuid;

#[derive(Debug)]
pub struct EFIVariableBootEvent {
    pub vendor_guid: Uuid,
    pub unicode_name: String,
    pub variable_data: Vec<u8>,
}

pub struct GrubEvent {
    pub cmd: String,
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
}
