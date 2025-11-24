// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

mod actor;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(feature = "faux_drives")]
mod faux;

/// Platform specific code.
mod platform {
    #[cfg(all(target_os = "linux", not(feature = "faux_drives")))]
    pub use super::linux::get_optical_drives;

    #[cfg(all(target_os = "linux", not(feature = "faux_drives")))]
    pub use super::linux::get_optical_drive;

    #[cfg(feature = "faux_drives")]
    pub use super::faux::get_optical_drives;

    #[cfg(feature = "faux_drives")]
    pub use super::faux::get_optical_drive;
}

use std::time::Duration;

use crate::Result;

pub use actor::{DriveActorHandle, DriveMessage};

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

/// Represents the state of the optical drive.
#[derive(Clone, Debug, PartialEq)]
pub enum DriveState {
    /// The drive information is no longer available.
    ///
    /// The main reasons a user may see this state is if they disconnect the drive after the
    /// application starts (e.g. unplug the USB device) or the worker node handling the drive
    /// stops communicating.
    Disconnected,

    /// The drive is waiting to begin a copy operation.
    Idle,

    /// The drive is in the process of copying a disc.
    Copying {
        /// The current stage of the copying process.
        stage: String,

        /// The task currently being performed.
        task: String,

        /// The percent complete (0 -> 0%, 1.0 -> 100%) of the current task.
        task_progress: f32,

        /// The subtask currently being performed.
        subtask: String,

        /// The percent complete (0 -> 0%, 1.0 -> 100%) of the current subtask.
        subtask_progress: f32,

        /// The length of time the copy operation has been running.
        elapsed_time: Duration,
    },

    /// The copy operation completed successfully.
    ///
    /// Once the copy operation completes, the drive will remain in this state until acknowledged
    /// which will reset the state back to `Idle`.
    Success,

    /// The copy operation failed or was cancelled.
    ///
    /// The drive will remain in this state until acknkowledged which will reset the state back to
    /// `Idle`.
    Failed {
        /// Brief description of what caused the failure.
        error: String,
    },
}

/// Represents an optical drive.
#[derive(Clone, Debug)]
pub struct Drive {
    /// The name assigned to the drive.
    ///
    /// This will be set to the serial number by default, but can be overwritten by the user.
    pub name: String,

    /// The device path of the drive, such as "/dev/sr0".
    pub path: String,

    /// The serial number of the optical drive.
    ///
    /// This may be a shortened version of the serial number assigned by the
    /// manufacturer.
    pub serial_number: String,

    /// The state of the disc in the optical drive.
    pub disc: DiscState,

    /// The state of the drive.
    pub state: DriveState,
}

impl From<OsDrive> for Drive {
    fn from(value: OsDrive) -> Self {
        Self {
            name: value.serial_number.clone(),
            path: value.path,
            serial_number: value.serial_number,
            disc: value.disc,
            state: DriveState::Idle,
        }
    }
}

/// The current status of the optical drive.
///
/// The status includes the state of the drive and the state of the disc inserted into the drive.
#[derive(Debug)]
pub struct DriveStatus {
    /// The state of the disc in the optical drive.
    pub disc: DiscState,

    /// The state of the drive.
    pub drive: DriveState,
}

impl DriveStatus {
    /// Create a [`DriveStatus`] instance.
    fn new(disc: DiscState, drive: DriveState) -> Self {
        Self { disc, drive, }
    }
}

/// Initialize the optical drive information for all available drives reported by the OS.
///
/// # Errors
///
/// - [`crate::Error::CommandIo`] or [`crate::Error::CommandReturnedErrorCode`] if the command to
///   to get the optical drive from the OS fails, or
/// - [`crate::Error::Serialization`] if the output from the OS cannot be parsed
pub fn init() -> Result<Vec<Drive>> {
    let drives = get_optical_drives()?.into_iter()
        .map(|d| d.into())
        .collect();
    Ok(drives)
}

/// Information about an optical drive as reported by the OS.
#[derive(Clone, Debug)]
struct OsDrive {
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
fn get_optical_drives() -> Result<Vec<OsDrive>> {
    let drives = platform::get_optical_drives()?;
    Ok(drives)
}

/// Gets the optical drive information for an optical drive with serial number `serial_number`.
///
/// Returns `None` if an optical drive cannot be found with the provided serial number. Returns an
/// error if something goes wrong when querying the operating system.
fn get_optical_drive(serial_number: &str) -> Result<Option<OsDrive>> {
    let drive = platform::get_optical_drive(serial_number)?;
    Ok(drive)
}

