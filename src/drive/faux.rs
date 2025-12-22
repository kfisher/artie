// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Faux implementation for development and testing purposes.

//! Faux implementation for development and testing purposes.

use std::sync::LazyLock;

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
    let drives = &*FAUX_DRIVES;
    Ok(drives.clone())
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
    for drive in &*FAUX_DRIVES {
        if drive.serial_number == serial_number {
            return Ok(Some(drive.clone()));
        }
    }

    Ok(None)
}

