// Copyright (c) 2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use sha2::{Digest as Sha2Digest, Sha256};
use std::io::{Cursor, Read, Seek};
use std::io::{Error as IoError, ErrorKind};
use strum::Display;

#[repr(u16)]
#[derive(TryFromPrimitive, Debug, PartialEq, Clone)]
pub enum TpmAlgorithmId {
    Error = 0x0000,         // TPM_ALG_ERROR
    RSA = 0x0001,           // TPM_ALG_RSA
    TDES = 0x0003,          // TPM_ALG_TDES
    SHA1 = 0x0004,          // TPM_ALG_SHA1
    HMAC = 0x0005,          // TPM_ALG_HMAC
    AES = 0x0006,           // TPM_ALG_AES
    MGF1 = 0x0007,          // TPM_ALG_MGF1
    KeyedHash = 0x0008,     // TPM_ALG_KEYEDHASH
    XOR = 0x000a,           // TPM_ALG_XOR
    SHA256 = 0x000b,        // TPM_ALG_SHA256
    SHA384 = 0x000c,        // TPM_ALG_SHA384
    SHA512 = 0x000d,        // TPM_ALG_SHA512
    Null = 0x0010,          // TPM_ALG_NULL
    SM3_256 = 0x0012,       // TPM_ALG_SM3_256
    SM4 = 0x0013,           // TPM_ALG_SM4
    RSASSA = 0x0014,        // TPM_ALG_RSASSA
    RSAES = 0x0015,         // TPM_ALG_RSAES
    RSAPSS = 0x0016,        // TPM_ALG_RSAPSS
    OAEP = 0x0017,          // TPM_ALG_OAEP
    ECDSA = 0x0018,         // TPM_ALG_ECDSA
    ECDH = 0x0019,          // TPM_ALG_ECDH
    ECDAA = 0x001a,         // TPM_ALG_ECDAA
    SM2 = 0x001b,           // TPM_ALG_SM2
    ECSchnorr = 0x001c,     // TPM_ALG_ECSCHNORR
    ECMQV = 0x001d,         // TPM_ALG_ECMQV
    Kdf1Sp800_56a = 0x0020, // TPM_ALG_KDF1_SP800_56A
    KDF2 = 0x0021,          // TPM_ALG_KDF2
    Kdf1Sp800_108 = 0x0022, // TPM_ALG_KDF1_SP800_108
    ECC = 0x0023,           // TPM_ALG_ECC
    SymCipher = 0x0025,     // TPM_ALG_SYMCIPHER
    Camellia = 0x0026,      // TPM_ALG_CAMELLIA
    SHA3_256 = 0x0027,      // TPM_ALG_SHA3_256
    SHA3_384 = 0x0028,      // TPM_ALG_SHA3_384
    SHA3_512 = 0x0029,      // TPM_ALG_SHA3_512
    CTR = 0x0040,           // TPM_ALG_CTR
    OFB = 0x0041,           // TPM_ALG_OFB
    CBC = 0x0042,           // TPM_ALG_CBC
    CFB = 0x0043,           // TPM_ALG_CFB
    ECB = 0x0044,           // TPM_ALG_ECB
}

#[repr(u32)]
#[derive(Debug, TryFromPrimitive, PartialEq, Display, Clone)]
pub enum TcgTpmEventType {
    PrebootCert = 0x00000000,          // EV_PREBOOT_CERT
    PostCode = 0x00000001,             // EV_POST_CODE
    NoAction = 0x00000003,             // EV_NO_ACTION
    Separator = 0x00000004,            // EV_SEPARATOR
    Action = 0x00000005,               // EV_ACTION
    EventTag = 0x00000006,             // EV_EVENT_TAG
    SCRTMContents = 0x00000007,        // EV_S_CRTM_CONTENTS
    SCRTMVersion = 0x00000008,         // EV_S_CRTM_VERSION
    CPUMicrocode = 0x00000009,         // EV_CPU_MICROCODE
    PlatformConfigFlags = 0x0000000a,  // EV_PLATFORM_CONFIG_FLAGS
    TableOfDevices = 0x0000000b,       // EV_TABLE_OF_DEVICES
    CompactHash = 0x0000000c,          // EV_COMPACT_HASH
    IPL = 0x0000000d,                  // EV_IPL
    IPLPartitionData = 0x0000000e,     // EV_IPL_PARTITION_DATA
    NonhostCode = 0x0000000f,          // EV_NONHOST_CODE
    NonhostConfig = 0x00000010,        // EV_NONHOST_CONFIG
    NonhostInfo = 0x00000011,          // EV_NONHOST_INFO
    OmitBootDeviceEvents = 0x00000012, // EV_OMIT_BOOT_DEVICE_EVENTS
    PostCode2 = 0x00000013,            // EV_POST_CODE2

    EfiEventBase = 0x80000000,               // EV_EFI_EVENT_BASE
    EfiVariableDriverConfig = 0x80000001,    // EV_EFI_VARIABLE_DRIVER_CONFIG
    EfiVariableBoot = 0x80000002,            // EV_EFI_VARIABLE_BOOT
    EfiBootServicesApplication = 0x80000003, // EV_EFI_BOOT_SERVICES_APPLICATION
    EfiBootServicesDriver = 0x80000004,      // EV_EFI_BOOT_SERVICES_DRIVER
    EfiRuntimeServicesDriver = 0x80000005,   // EV_EFI_RUNTIME_SERVICES_DRIVER
    EfiGPTEvent = 0x80000006,                // EV_EFI_GPT_EVENT
    EfiAction = 0x80000007,                  // EV_EFI_ACTION
    EfiPlatformFirmwareBlob = 0x80000008,    // EV_EFI_PLATFORM_FIRMWARE_BLOB
    EfiHandoffTables = 0x80000009,           // EV_EFI_HANDOFF_TABLES
    EfiPlatformFirmwareBlob2 = 0x8000000a,   // EV_EFI_PLATFORM_FIRMWARE_BLOB2
    EfiHandoffTables2 = 0x8000000b,          // EV_EFI_HANDOFF_TABLES2
    EfiVariableBoot2 = 0x8000000c,           // EV_EFI_VARIABLE_BOOT2
    EfiGPTEvent2 = 0x8000000d,               // EV_EFI_GPT_EVENT2
    EfiHCRTMEvent = 0x80000010,              // EV_EFI_HCRTM_EVENT
    EfiVariableAuthority = 0x800000e0,       // EV_EFI_VARIABLE_AUTHORITY
    EfiSPDMFirmwareBlob = 0x800000e1,        // EV_EFI_SPDM_FIRMWARE_BLOB
    EfiSPDMFirmwareConfig = 0x800000e2,      // EV_EFI_SPDM_FIRMWARE_CONFIG
    EfiSPDMDevicePolicy = 0x800000e3,        // EV_EFI_SPDM_DEVICE_POLICY
    EfiSPDMDeviceAuthority = 0x800000e4,     // EV_EFI_SPDM_DEVICE_AUTHORITY
}

impl TcgTpmEventType {
    pub fn is_efi_boot_variable(&self) -> bool {
        matches!(
            self,
            TcgTpmEventType::EfiVariableDriverConfig
                | TcgTpmEventType::EfiVariableBoot
                | TcgTpmEventType::EfiVariableBoot2
        )
    }
}

#[repr(u32)]
#[derive(Debug, TryFromPrimitive, PartialEq)]
pub enum EvePcrIndex {
    GrubPcr = 8,
    GrubInitrdPcr = 9,
    RootFsPcr = 13,
    ConfigPcr = 14,
}

impl TpmAlgorithmId {
    pub fn get_byte_size(&self) -> Result<usize> {
        match self {
            TpmAlgorithmId::SHA1 => Ok(20),
            TpmAlgorithmId::SHA256 => Ok(32),
            TpmAlgorithmId::SHA384 => Ok(48),
            TpmAlgorithmId::SHA512 => Ok(64),
            TpmAlgorithmId::SM3_256 => Ok(32),
            _ => Err(anyhow!("Unsupported algorithm for digest")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TcgRawTpmEvent {
    pub pcr_index: u32,
    pub event_type: TcgTpmEventType,
    pub digests: Vec<Digest>,
    pub event_data: Vec<u8>,
}

// Implement PartialEq for TcgTpmEvent
// we do not need to cpmpare event_data
// becasue digest is calculated from event_data
// we need to compare only one digest from the list
// impl PartialEq for TcgTpmEvent {
//     fn eq(&self, other: &Self) -> bool {
//         self.pcr_index == other.pcr_index
//             && self.event_type == other.event_type
//             && self.digests[0] == other.digests[0]
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub struct Digest {
    pub algorithm_id: TpmAlgorithmId,
    pub digest: Vec<u8>,
}

#[repr(u32)]
#[derive(TryFromPrimitive, Debug, PartialEq)]
enum PlatformType {
    Unknown = 0,
    BIOS = 1,
    EFI = 2,
}

#[derive(Debug)]
struct LogSpec {
    major: u8,
    minor: u8,
    platform_type: PlatformType,
    digest_length: Option<Vec<DigestSize>>,
}

impl LogSpec {
    fn is_efi_2(&self) -> bool {
        self.platform_type == PlatformType::EFI && self.major == 2
    }
}

#[derive(Debug)]
struct DigestSize {
    algorithm_id: TpmAlgorithmId,
    size: usize,
}

impl Digest {
    pub fn new_sha256(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        Self {
            algorithm_id: TpmAlgorithmId::SHA256,
            digest: digest.to_vec(),
        }
    }
}

impl TcgRawTpmEvent {
    pub fn new(pcr_index: u32, event_type: TcgTpmEventType, event_data: Vec<u8>) -> Self {
        let digests = Digest::new_sha256(&event_data);
        Self {
            pcr_index,
            event_type,
            digests: vec![digests],
            event_data,
        }
    }
}

impl TryFrom<TcgRawTpmEvent> for LogSpec {
    type Error = anyhow::Error;

    fn try_from(event: TcgRawTpmEvent) -> std::result::Result<Self, Self::Error> {
        if event.event_type != TcgTpmEventType::NoAction {
            return Err(anyhow!("Not a NoAction event"));
        }

        // read signature field 16 bytes
        let mut cursor = Cursor::new(&event.event_data);
        let mut signature = [0u8; 16];

        cursor
            .read_exact(&mut signature)
            .context("cannot read signature bytes")?;

        // convert to string. signature is padded with 0x0
        let signature = String::from_utf8_lossy(&signature)
            .trim_end_matches(|c| c == char::from_u32(0).unwrap())
            .to_string();

        match signature.as_str() {
            "Spec ID Event00" => {
                // read spec id event data
                let _platform_class = cursor.read_u32::<LittleEndian>()?;
                let minor = cursor.read_u8()?;
                let major = cursor.read_u8()?;

                Ok(LogSpec {
                    major,
                    minor,
                    platform_type: PlatformType::BIOS,
                    digest_length: None,
                })
            }
            "Spec ID Event02" => {
                let _platform_class = cursor.read_u32::<LittleEndian>()?;
                let minor = cursor.read_u8()?;
                let major = cursor.read_u8()?;

                Ok(LogSpec {
                    major,
                    minor,
                    platform_type: PlatformType::EFI,
                    digest_length: None,
                })
            }
            "Spec ID Event03" => {
                let _platform_class = cursor.read_u32::<LittleEndian>()?;
                let minor = cursor.read_u8()?;
                let major = cursor.read_u8()?;
                cursor.seek_relative(2)?; // skip 2 bytes errata and uintn_size

                let digest_count = cursor.read_u32::<LittleEndian>()?;
                let mut digest_length_list = Vec::with_capacity(digest_count as usize);

                for _ in 0..digest_count {
                    let algorithm_id = cursor.read_u16::<LittleEndian>()?;
                    let algorithm_id = TpmAlgorithmId::try_from(algorithm_id)?;
                    let digest_size = cursor.read_u16::<LittleEndian>()?;
                    digest_length_list.push(DigestSize {
                        algorithm_id,
                        size: digest_size as usize,
                    });
                }

                Ok(LogSpec {
                    major,
                    minor,
                    platform_type: PlatformType::EFI,
                    digest_length: Some(digest_length_list),
                })
            }
            _ => Err(anyhow!(format!("Unsupported signature {}", signature))),
        }
    }
}

// struct SpecId00Event {}

#[derive(Debug, Clone)]
pub struct TcgTpmLog {
    pub events: Vec<TcgRawTpmEvent>,
}

#[derive(Debug, Clone)]
pub struct TcgTpmEventRef<'a> {
    pub original_index: usize,
    pub event: &'a TcgRawTpmEvent,
}

impl PartialEq for TcgTpmEventRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        *self.event == *other.event
    }
}

impl TcgTpmLog {
    pub fn from_events(events: Vec<TcgRawTpmEvent>) -> Self {
        Self { events }
    }

    pub fn from_slice(data: &[u8]) -> Result<Self> {
        let events = Self::deserialize_tpm_event_log(data)?;
        Ok(Self { events })
    }

    fn read_event(cursor: &mut Cursor<&[u8]>, efi_2_0: bool) -> Result<TcgRawTpmEvent> {
        let pcr_index = cursor.read_u32::<LittleEndian>()?;
        let event_type = cursor.read_u32::<LittleEndian>()?;
        let event_type = TcgTpmEventType::try_from(event_type).map_err(|e| {
            IoError::new(
                ErrorKind::InvalidData,
                format!("Failed to convert to TpmEventType: {}", e),
            )
        })?;

        let digests = if efi_2_0 {
            // Read digest count and parse each digest
            let digest_count = cursor.read_u32::<LittleEndian>()?;
            let mut digests = Vec::with_capacity(digest_count as usize);

            for _ in 0..digest_count {
                let algorithm_id = cursor.read_u16::<LittleEndian>()?;

                let algorithm_id = TpmAlgorithmId::try_from(algorithm_id).map_err(|e| {
                    IoError::new(
                        ErrorKind::InvalidData,
                        format!("Failed to convert to TpmAlgorithmId: {}", e),
                    )
                })?;

                let digest_size = algorithm_id.get_byte_size()?; // Implement this based on TPM specs
                let mut digest = vec![0u8; digest_size];
                cursor.read_exact(&mut digest)?;

                digests.push(Digest {
                    algorithm_id,
                    digest,
                });
            }
            digests
        } else {
            let mut digests = Vec::with_capacity(1);
            // read sha1 digest
            let digest_size = TpmAlgorithmId::SHA1.get_byte_size()?;
            let mut digest = vec![0u8; digest_size];
            cursor.read_exact(&mut digest)?;
            digests.push(Digest {
                algorithm_id: TpmAlgorithmId::SHA1,
                digest,
            });
            digests
        };

        // Read event data
        let event_data_size = cursor.read_u32::<LittleEndian>()?;

        let mut event_data = vec![0u8; event_data_size as usize];
        cursor.read_exact(&mut event_data)?;

        Ok(TcgRawTpmEvent {
            pcr_index,
            event_type,
            digests,
            event_data,
        })
    }

    fn deserialize_tpm_event_log(data: &[u8]) -> Result<Vec<TcgRawTpmEvent>> {
        let mut cursor = Cursor::new(data);
        let mut events = Vec::new();

        // the very first event is always a Spec event in 'old' format wit honly SHA1 digest
        let spec_event = Self::read_event(&mut cursor, false)?;
        let log_spec = LogSpec::try_from(spec_event)?;

        // Loop until all data is read
        while cursor.position() < data.len() as u64 {
            let event = Self::read_event(&mut cursor, log_spec.is_efi_2())?;
            events.push(event);
        }

        Ok(events)
    }

    pub fn events_for_pcr_ref(&self, pcr_index: u32) -> Vec<TcgTpmEventRef> {
        self.events
            .iter()
            .enumerate()
            .filter_map(|(index, e)| {
                if e.pcr_index == pcr_index {
                    Some(TcgTpmEventRef {
                        original_index: index,
                        event: e,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn into_events_for_pcr(self, pcr_index: u32) -> Vec<TcgRawTpmEvent> {
        self.events
            .into_iter()
            .filter(|e| e.pcr_index == pcr_index)
            .collect()
    }
}
