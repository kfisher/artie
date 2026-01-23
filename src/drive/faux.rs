// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Faux implementation for development and testing purposes.

//! Faux implementation for development and testing purposes.

use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::Result;

use super::{DiscState, OsOpticalDrive};

static FAUX_DRIVES: LazyLock<Vec<OsOpticalDrive>> = LazyLock::new(|| {
    vec![
        OsOpticalDrive {
            path: String::from("/dev/fx0"),
            serial_number: String::from("FAUX0001"),
            disc: DiscState::None,
        },
        OsOpticalDrive {
            path: String::from("/dev/fx1"),
            serial_number: String::from("FAUX0002"),
            disc: DiscState::Inserted {
                label: String::from("FAUX_MOVIE"),
                uuid: String::from("00000000-0000-0000-0000-000000000000"),
            },
        },
    ]
});

/// Gets the optical drive information for all available optical drives.
///
/// This is a fake implementation only meant for development and testing where the development
/// system may not have optical drives or when it might not be desireable to use actual drives
/// such as automated tests.
pub fn get_optical_drives() -> Result<Vec<OsOpticalDrive>> {
    let drives = get_faux_optical_drives()?.into_iter()
        .map(|fd| fd.into_os_drive())
        .collect();
    Ok(drives)
}

/// Gets the optical drive information for an optical drive with serial number
/// `serial_number`.
///
/// Returns `None` if an optical drive cannot be found with the provided serial
/// number. Returns an error if something goes wrong when querying the operating
/// system.
///
/// This is a fake implementation only meant for development and testing where the development
/// system may not have optical drives or when it might not be desireable to use actual drives
/// such as automated tests.
pub fn get_optical_drive(serial_number: &str) -> Result<Option<OsOpticalDrive>> {
    let drive = get_faux_optical_drive(serial_number)?
        .map(|fd| fd.into_os_drive());
    Ok(drive)
}

const FAUX_DRIVES_DIR: &str = "./faux_drives";

#[derive(Clone, Default, Deserialize, Serialize)]
struct FauxDisc {
    pub label: String,
    pub uuid: String,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct FauxDrive {
    pub name: String,
    pub path: String,
    pub serial_number: String,
    pub disc: Option<FauxDisc>,
}

impl FauxDrive {
    pub fn into_os_drive(self) -> OsOpticalDrive {
        OsOpticalDrive { 
            path: self.path,
            serial_number: self.serial_number,
            disc: match self.disc {
                Some(disc) => DiscState::Inserted { label: disc.label, uuid: disc.uuid },
                None => DiscState::None,
            },
        }
    }
}

fn get_faux_optical_drives() -> Result<Vec<FauxDrive>> {
    let drives_dir = PathBuf::from(FAUX_DRIVES_DIR);
    
    if !drives_dir.exists() {
        return Ok(Vec::new());
    }

    let mut drives = Vec::new();
    
    for entry in fs::read_dir(&drives_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let contents = fs::read_to_string(&path).unwrap();
            let drive: FauxDrive = serde_json::from_str(&contents).unwrap();
            drives.push(drive);
        }
    }
    
    Ok(drives)
}

fn get_faux_optical_drive(serial_number: &str) -> Result<Option<FauxDrive>> {
    let drives = get_faux_optical_drives()?;
    Ok(drives.into_iter().find(|drive| drive.serial_number == serial_number))
}

