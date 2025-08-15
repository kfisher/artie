// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for interfacing with the operating system in order to get
//! information about optical drives.

#[cfg(target_os = "linux")]
mod linux;

/// Platform specific code.
mod platform {
    #[cfg(target_os = "linux")]
    pub use super::linux::get_optical_drive;
}

/// Specifies the errors that can occur when performing optical drive
/// operations.
pub enum Error {
    /// Indicates an error occurred while attempting to run an external command.
    CommandFailed(std::io::Error),

    /// Indicates an external command existed with an error code.
    CommandReturnedErrorCode(i32),

    /// An error that can occur when converting raw bytes from an external
    /// command's standard output or standard error to a string, or vice-versa.
    ConversionError(std::string::FromUtf8Error),

    /// An error occurred while processing the JSON output from an external
    /// command.
    JsonError(serde_json::Error),
}

/// Represents the state of the optical drive's disc.
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

/// Gets the optical drive information for an optical drive with serial number
/// `serial_number`.
///
/// Returns `None` if an optical drive cannot be found with the provided serial
/// number. Returns an error if something goes wrong when querying the operating
/// system.
pub fn get_optical_drive(serial_number: &str) -> Result<Option<OpticalDrive>, Error> {
    platform::get_optical_drive(serial_number)
}
