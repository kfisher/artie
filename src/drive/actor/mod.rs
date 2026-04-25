// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Actors for interfacing with optical drives.

pub mod local;
pub mod manager;

use tokio::sync::mpsc;

use tokio_util::sync::CancellationToken;

use makemkv::{CommandOutput, CopyCommandOutput, InfoCommandOutput};

use crate::actor::Response;
use crate::drive::{OpticalDrive, OpticalDriveState, OsOpticalDrive};
use crate::models::{CopyParamaters, MediaLocation};

pub use crate::drive::data::{FormData, FormDataUpdate};

/// Optical drive actor requests.
#[derive(Debug)]
pub enum DriveRequest {
    /// Begin copying a disc.
    BeginCopyDisc {
        params: CopyParamaters,
        response: Response<()>,
    },

    /// Cancel an in-progress copy operation.
    CancelCopyDisc {
        response: Response<()>,
    },

    /// Get the current status of an optical drive.
    GetStatus {
        response: Response<OpticalDrive>,
    },

    /// Get the last saved values for a drive's copy parameters.
    ReadFormData {
        response: Response<FormData>,
    },

    /// Reset the drive state back to idle.
    ///
    /// Should only be requested if the drive state is currently in `Success` or `Failed`. Will
    /// result in an error if in any other state.
    Reset {
        response: Response<()>,
    },

    /// Request to run the MakeMKV info command to gather information about the titles on the disc.
    RunMakeMkvInfo {
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: String,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: Response<InfoCommandOutput>,
    },

    /// Request to run the MakeMKV copy command to copy titles from the disc to the file system.
    RunMakeMkvCopy {
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: String,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: Response<CopyCommandOutput>,
    },

    /// Update the copy parameters stored in the drive's persistent data.
    SaveFormData {
        data: FormDataUpdate,
        response: Response<()>,
    },

    /// Updates the current state of the drive.
    ///
    /// This will update the drive status information based on an in-progress copy operation or an
    /// operation that completed, failed, or was cancelled.
    UpdateFromCopy {
        state: OpticalDriveState,
        response: Response<()>,
    },

    /// Updates the current state of the drive.
    ///
    /// This will update the drive status information based on what is reported by the OS. It is 
    /// mainly meant for use within the drive module only which is why there isn't a corresponding
    /// helper function.
    ///
    /// If `info` is `None`, then the information was unavailable without any errors being
    /// reported. This most likely means the drive is disconnected and will be treated as such.
    UpdateFromOs {
        info: Option<OsOpticalDrive>,
        response: Response<()>,
    },
}

/// Optical drive manager requests
#[derive(Debug)]
pub enum ManagerRequest {
    /// Get list of drive serial numbers.
    ///
    /// This will return the serial numbers for all optical drives that have an associated drive
    /// actor. This includes local drives and those on remote worker nodes. Use the appropriate
    /// drive specific request to get details about the drives.
    GetDrives {
        response: Response<Vec<String>>,
    },
}
