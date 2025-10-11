// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Faux implementation for development and testing purposes.

use std::sync::LazyLock;

use crate::{OpticalDrive, Result};

static FAUX_DRIVES: LazyLock<Vec<OpticalDrive>> = LazyLock::new(|| {
    vec![
        OpticalDrive {
            path: String::from("/dev/fx0"),
            serial_number: String::from("FAUX0001"),
            disc: crate::DiscState::None,
        },
        OpticalDrive {
            path: String::from("/dev/fx1"),
            serial_number: String::from("FAUX0002"),
            disc: crate::DiscState::Inserted {
                label: String::from("FAUX_MOVIE"),
                uuid: String::from("00000000-0000-0000-0000-000000000000"),
            },
        },
    ]
});

/// Gets the optical drive information for an optical drive with serial number
/// `serial_number`.
///
/// Returns `None` if an optical drive cannot be found with the provided serial
/// number. Returns an error if something goes wrong when querying the operating
/// system.
///
/// This is the Linux specific implementation.
pub fn get_optical_drive(serial_number: &str) -> Result<Option<OpticalDrive>> {
    for drive in &*FAUX_DRIVES {
        if drive.serial_number == serial_number {
            return Ok(Some(drive.clone()));
        }
    }

    Ok(None)
}
