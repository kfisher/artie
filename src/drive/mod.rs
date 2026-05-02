// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles all optical drive operations.
//!
//! # Actors
//!
//! The drive module consists of several types of application actors. The drive manager actor is
//! responsible for managing the other drive actor instances. It also serves as the broker for
//! drive related requests coming from the message bus. The following functions can be used to make
//! requests to the manager:
//!
//! - [`get_drives`] - Get list of available optical drives.
//!
//! There are several types of drive actors depending on if the drive is connected to the host the
//! application is running on and if the application is the control node or a worker node.
//! Regardless of the type, all drive actor requests are made using the following functions:
//!
//! - [`begin_copy`] - Starts a copy operation.
//! - [`cancel_copy`] - Cancels a running copy operation.
//! - [`get`] - Get details about an optical drive and its current state.
//! - [`read_form_data`] - Read the saved copy parameter values.
//! - [`reset`] - Resets the drive back to the `Idle` state after a successful or failed copy
//!   operation.
//! - [`save_form_data`] - Saves the current copy parameters.
//!
//! # Initialization
//!
//! The drive manager can be initialized by calling [`init`]. It will handle initializing the drive
//! actor instances.

mod actor;
mod copy;
mod data;
mod makemkv;
mod manager;
mod monitor;
mod worker;

#[cfg(all(target_os = "linux", not(feature = "faux_drives")))]
mod linux;

#[cfg(feature = "faux_drives")]
mod faux;

use std::time::Duration;

use serde::{Deserialize, Serialize};

use tokio::sync::oneshot;

use crate::{Error, Result};
use crate::bus;
use crate::models::CopyParamaters;

pub use data::{FormData, FormDataUpdate};
pub use manager::init;

use actor::DriveRequest;
use manager::ManagerRequest;

/// Handle used to communicate with the drive actors and manager.
pub type Handle = crate::actor::Handle<Message>;

/// Represents the state of the optical drive's disc.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DiscState {
    /// No disc is inserted in the optical drive.
    None,

    /// A disc is inserted in the optical drive.
    ///
    /// `label` is the label of the disc. `uuid` is a unique identifier assigned
    /// to the disc by the OS.
    Inserted { 
        /// The disc label.
        label: String,

        /// Unique identifier assigned to the disc by the OS.
        uuid: String 
    },
}

/// Message for sending requests to a drive actor or the drive manager.
///
/// The drive manager is what processes all drive related messages. If the message is meant for a
/// specific drive, the message will be forwarded by the manager to that drive; Otherwise, the
/// manager will process the message.
#[derive(Debug)]
pub enum Message {
    /// Message type for sending requests to a drive actor.
    Drive {
        serial_number: String,
        request: DriveRequest,
    },

    /// Message type for sending requests to the drive manager.
    Manager {
        request: ManagerRequest,
    },
}

impl Message {
    /// Consume the message and return the drive request.
    ///
    /// # Args
    ///
    /// `serial_number`:  The serial number of the drive associated with the actor processing the
    /// request. Used to verify the serial number matches the message serial number.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidDriveRequest`] if this message is not a drive message or if the message
    /// serial number does not match the provided drive's serial number.
    pub fn drive_request(self, serial_number: &str) -> Result<DriveRequest> {
        let Message::Drive { serial_number: target_serial_number, request } = self else {
            return Err(Error::InvalidDriveRequest);
        };

        if serial_number != target_serial_number {
            return Err(Error::InvalidDriveRequest);
        }

        Ok(request)
    }

}

/// Represents the state of the optical drive.
#[derive(Clone, Debug, PartialEq)]
pub enum OpticalDriveState {
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

impl OpticalDriveState {
    /// Get the name (or label) of the state.
    pub fn name(&self) -> &'static str {
        match self {
            OpticalDriveState::Disconnected => "Disconnected",
            OpticalDriveState::Idle => "Idle",
            OpticalDriveState::Copying { .. } => "Copying",
            OpticalDriveState::Success => "Success",
            OpticalDriveState::Failed { .. } => "Failed",
        }
    }
}

/// Represents an optical drive.
///
/// This is the optical drive data returned by the associated drive actor when [`get`] is called.
#[derive(Clone, Debug)]
pub struct OpticalDrive {
    /// The user defined name of the drive.
    ///
    /// The name will initially be set to the drive's serial number. The name can be overwritten by
    /// the user to be whatever they want to make it easier for them to identify the drive.
    pub name: String,

    /// The device path of the drive, such as "/dev/sr0".
    pub path: String,

    /// The serial number of the optical drive.
    ///
    /// This may be a shortened version of the serial number assigned by the manufacturer.
    pub serial_number: String,

    /// The hostname of the system the drive is installed in.
    pub hostname: String,

    /// The state of the disc in the optical drive.
    pub disc: DiscState,

    /// The state of the drive.
    ///
    /// This is the state within the context of this application which is mainly if its idle,
    /// copying, etc., not the state of the drive hardware itself.
    pub state: OpticalDriveState,
}

impl OpticalDrive {
    /// Create a new optical drive instance for a disconnected drive.
    ///
    /// This can be used when the OS does not have a drive connected with the provided serial
    /// number or the worker node the drive is connected to is disconnected.
    pub fn disconnected(serial_number: &str) -> Self {
        Self {
            name: serial_number.to_owned(),
            path: String::default(),
            serial_number: serial_number.to_owned(),
            hostname: String::default(),
            disc: DiscState::None,
            state: OpticalDriveState::Disconnected,
        }
    }

    /// Create a new optical drive instance from drive information provided by the OS.
    ///
    /// By default, the `name` will be the drive's serial number and the state will be the
    /// disconnected state.
    ///
    /// # Args
    ///
    /// `drive`:  Information about about the optical drive as reported by the OS.
    pub fn from_os(drive: OsOpticalDrive) -> Self {
        Self {
            name: drive.serial_number.clone(),
            path: drive.path,
            serial_number: drive.serial_number,
            hostname: drive.hostname,
            disc: drive.disc,
            state: OpticalDriveState::Disconnected,
        }
    }

    /// Creates an [`OsOpticalDrive`] instance from this drive instance.
    pub fn os_drive(&self) -> OsOpticalDrive {
        OsOpticalDrive {
            path: self.path.clone(),
            serial_number: self.serial_number.clone(),
            disc: self.disc.clone(),
            hostname: self.hostname.clone(),
        }
    }
}

/// Information reported by the operating system for the optical drive.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OsOpticalDrive {
    /// The device path of the drive, such as "/dev/sr0".
    pub path: String,

    /// The serial number of the optical drive.
    ///
    /// This may be a shortened version of the serial number assigned by the manufacturer.
    pub serial_number: String,

    /// The state of the disc in the optical drive.
    pub disc: DiscState,

    /// The hostname of the system the drive is installed in.
    pub hostname: String,
}

/// Begin copying a disc.
///
/// # Args
///
/// `bus`:  Handle for sending messages to the drive actor.
///
/// `serial_number`:  Serial number of the optical drive.
///
/// `params`:  The parameters for the copy operation such as the title, release year, or disc
/// number of the disc being copied.
///
/// # Errors
///
/// TODO
pub async fn begin_copy(
    bus: &bus::Handle,
    serial_number: &str,
    params: CopyParamaters
) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Drive {
        serial_number: serial_number.to_owned(),
        request: DriveRequest::BeginCopyDisc { params, response: tx },
    };
    bus.send(msg).await?;
    rx.await?
}

/// Cancel an in-progress copy operation.
///
/// # Args
///
/// `bus`:  Handle for sending messages to the drive actor.
///
/// `serial_number`:  Serial number of the optical drive.
///
/// # Errors
///
/// [`crate::Error::StdIo`] or [`crate::Error::SerdeJson`] if the file containing the data exists,
/// but cannot be read or parsed. The file not existing is not considered an error which will
/// result in default values being returned.
///
/// TODO: General Send Failures
/// TODO: Worker
pub async fn cancel_copy(bus: &bus::Handle, serial_number: &str) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Drive {
        serial_number: serial_number.to_owned(),
        request: DriveRequest::CancelCopyDisc { response: tx },
    };
    bus.send(msg).await?;
    rx.await?
}

/// Get a list of serial numbers for all known optical drives.
///
/// The serial numbers can be used in any of the drive actor requests such as [`get`] to get more
/// detailed information about the drive.
///
/// # Args
///
/// `bus`:  Handle for sending messages to the drive manager.
///
/// # Errors
///
/// TODO
pub async fn get_drives(bus: &bus::Handle) -> Result<Vec<String>> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Manager { 
        request: ManagerRequest::GetDrives { response: tx }
    };
    bus.send(msg).await?;
    rx.await?
}

/// Get the current status of an optical drive.
///
/// # Args
///
/// `bus`:  Handle for sending messages to the drive actor.
///
/// `serial_number`:  Serial number of the optical drive.
///
/// # Errors
///
/// TODO
pub async fn get(bus: &bus::Handle, serial_number: &str) -> Result<OpticalDrive> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Drive {
        serial_number: serial_number.to_owned(),
        request: DriveRequest::GetStatus { response: tx },
    };
    bus.send(msg).await?;
    rx.await?
}

/// Get the last saved values for a drive's copy parameters.
///
/// # Args
///
/// `bus`:  Handle for sending messages to the drive actor.
///
/// `serial_number`:  Serial number of the driveoptical .
///
/// # Errors
///
/// TODO
pub async fn read_form_data(bus: &bus::Handle, serial_number: &str) -> Result<FormData> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Drive {
        serial_number: serial_number.to_owned(),
        request: DriveRequest::ReadFormData { response: tx },
    };
    bus.send(msg).await?;
    rx.await?
}

/// Reset a drive's state back to idle.
///
/// # Args
///
/// `bus`:  Handle for sending messages to the drive actor.
///
/// `serial_number`:  Serial number of the optical drive.
///
/// # Errors
///
/// TODO
///
/// [`crate::Error::InvalidDriveState`] if the drive is not in the `Success` or `Failed` state.
pub async fn reset(bus: &bus::Handle, serial_number: &str) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Drive {
        serial_number: serial_number.to_owned(),
        request: DriveRequest::Reset { response: tx },
    };
    bus.send(msg).await?;
    rx.await?
}

/// Updated the saved values for the drive's copy parameters.
///
/// # Args
///
/// `bus`:  Handle for sending messages to the drive actor.
///
/// `serial_number`:  Serial number of the drive whose state should be reset.
///
/// # Errors
///
/// TODO
pub async fn save_form_data(
    bus: &bus::Handle,
    serial_number: &str,
    data: FormDataUpdate
) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Drive {
        serial_number: serial_number.to_owned(),
        request: DriveRequest::SaveFormData { data, response: tx },
    };
    bus.send(msg).await?;
    rx.await?
}

/// Update the status of a drive based off information reported by the OS.
///
/// `bus`:  Handle for sending messages to the drive actor.
///
/// `serial_number`:  Serial number of the drive whose status is being updated.
///
/// `drive`:  The optical drive information reported by the OS.
pub async fn update_from_os(bus: &bus::Handle, drive: OsOpticalDrive,) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Drive {
        serial_number: drive.serial_number.clone(),
        request: DriveRequest::UpdateFromOs { drive, response: tx },
    };
    bus.send(msg).await?;
    rx.await?
}

/// Gets the optical drive information for all available optical drives.
///
/// # Errors
///
/// The specific errors depend on the platform implementation.
fn get_optical_drives() -> Result<Vec<OsOpticalDrive>> {
    let drives = platform::get_optical_drives()?;
    Ok(drives)
}

/// Gets the optical drive information for an optical drive with serial number `serial_number`.
///
/// This is the Linux specific implementation.
///
/// # Returns
///
/// Returns `Some` if a drive can be found with the provided serial number or `None` otherwise.
///
/// # Errors
///
/// The specific errors depend on the platform implementation.
fn get_optical_drive(serial_number: &str) -> Result<Option<OsOpticalDrive>> {
    let drive = platform::get_optical_drive(serial_number)?;
    Ok(drive)
}

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

#[cfg(test)]
mod tests {
    // TODO
}
