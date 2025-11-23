// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles interactions with optical drives.

#[cfg(target_os = "linux")]
mod linux;

#[cfg(feature = "faux_drives")]
mod faux;

/// Platform specific code.
mod platform {
    #[cfg(target_os = "linux")]
    pub use super::linux::get_optical_drives;

    #[cfg(target_os = "linux")]
    pub use super::linux::get_optical_drive;
}

use crate::Result;

/// Represents the state of the optical drive's disc.
#[derive(Clone, Debug, PartialEq)]
pub enum DiscState {
    /// No disc is inserted in the optical drive.
    None,

    /// A disc is inserted in the optical drive.
    ///
    /// `label` is the label of the disc. `uuid` is a unique identifier assigned
    /// to the disc by the OS.
    Inserted { label: String, uuid: String },
}

/// Represents an optical drive.
#[derive(Clone, Debug, PartialEq)]
pub struct OpticalDrive {
    /// The device path of the drive, such as "/dev/sr0".
    pub path: String,

    /// The serial number of the optical drive.
    ///
    /// This may be a shortened version of the serial number assigned by the
    /// manufacturer.
    pub serial_number: String,

    /// The state of the disc in the optical drive.
    pub disc: DiscState,
}

/// Gets the optical drive information for all available optical drives.
pub fn get_optical_drives() -> Result<Vec<OpticalDrive>> {
    #[cfg(feature = "faux_drives")]
    {
        let mut drives = platform::get_optical_drives()?;
        drives.extend(faux::get_optical_drives());
        Ok(drives)
    }

    #[cfg(not(feature = "faux_drives"))]
    {
        let drives = platform::get_optical_drives()?;
        Ok(drives)
    }
}

/// Gets the optical drive information for an optical drive with serial number `serial_number`.
///
/// Returns `None` if an optical drive cannot be found with the provided serial number. Returns an
/// error if something goes wrong when querying the operating system.
pub fn get_optical_drive(serial_number: &str) -> Result<Option<OpticalDrive>> {
    let drive = platform::get_optical_drive(serial_number)?;

    #[cfg(feature = "faux_drives")]
    if drive.is_none() {
        return Ok(faux::get_optical_drive(serial_number));
    }

    Ok(drive)
}
