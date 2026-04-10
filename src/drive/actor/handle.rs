// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use tokio_util::sync::CancellationToken;

use makemkv::{CommandOutput, CopyCommandOutput, InfoCommandOutput};

use crate::Result;
use crate::error::{ChannelError, Error};
use crate::drive::{OpticalDriveState, OpticalDriveStatus};
use crate::drive::actor::DriveActorMessage;
use crate::drive::data::{FormData, FormDataUpdate};
use crate::models::{CopyParamaters, MediaLocation};

/// Handle used to communicate with a drive actor.
///
/// There is a single drive actor handle  type for communicating with any of the drive actor types.
/// This allows the rest of the application to interface with the drives the same way regardless of
/// what machine the drive is actually on.
///
/// The handles are not created directly. Instead they are created by the actor creation functions
/// which exists for each type of drive actor. See:
///
/// - [`crate::drive::actor::local::create_actor`]
#[derive(Clone, Debug)]
pub struct DriveActorHandle {
    /// Transmission end of the channel used to send requests to the actor.
    tx: mpsc::Sender<DriveActorMessage>,
}

impl DriveActorHandle {
    /// Create a new [`DriveActorHandle`] instance.
    pub fn new(tx: mpsc::Sender<DriveActorMessage>) -> Self {
        Self { tx }
    }

    /// Cancels a copy operation currently in progress.
    pub async fn cancel_copy_disc(&self) -> Result<()> {
        let msg = DriveActorMessage::CancelCopyDisc;
        self.tx.send(msg).await.map_err(send_error)
    }

    /// Begins copying the disc in the optical drive.
    pub async fn copy_disc(&self, copy_parameters: CopyParamaters) -> Result<()> {
        let msg = DriveActorMessage::CopyDisc { parameters: copy_parameters };
        self.tx.send(msg).await.map_err(send_error)
    }

    /// Get the form data from the drive's persistent storage.
    pub async fn get_form_data(&self) -> Result<FormData> {
        let (tx, rx) = oneshot::channel();

        let msg = DriveActorMessage::GetFormData { response: tx };

        self.tx.send(msg).await
            .map_err(send_error)?;

        rx.await.map_err(oneshot_recv_error)
    }

    /// Get the current status of the optical drive.
    pub async fn get_status(&self) -> Result<OpticalDriveStatus> {
        let (tx, rx) = oneshot::channel();

        let msg = DriveActorMessage::GetStatus { response: tx };

        self.tx.send(msg).await
            .map_err(send_error)?;

        rx.await.map_err(oneshot_recv_error)
    }

    /// Run the MakeMKV info command to gather information about a disc's titles.
    ///
    /// **Note** This is called as part of the [`DriveActorHandle::copy_disc`] request.
    ///
    /// `command_output`:  Channel used by the MakeMKV command to relay output from the command as
    /// well as progress information.
    ///
    /// `device_path`:  Device path (or name) of the optical drive to perform the operation on
    /// (e.g. "/dev/sr0").
    ///
    /// `log_file`:  The file location where the output of the command should be logged to.
    ///
    /// `cancellation_token`:  Cancellation token used to cancel the copy operation. It is assumed
    /// that the token is not already cancelled.
    ///
    /// # Errors
    ///
    /// [`Error::MakeMKV`] if an error occures while running the MakeMKV command.
    ///
    /// [`Error::DriveActorChannel`] if an error occures when attempting to send information thru
    /// a channel.
    pub async fn run_makemkv_info(
        &self,
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: &str,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
    ) -> Result<InfoCommandOutput> {
        let (tx, rx) = oneshot::channel();

        let msg = DriveActorMessage::RunMakeMkvInfo {
            command_output,
            device_path: device_path.to_owned(),
            log_file,
            cancellation_token,
            response: tx,
        };

        self.tx.send(msg).await
            .map_err(send_error)?;

        match rx.await {
            Ok(result) => result,
            Err(error) => Err(oneshot_recv_error(error)),
        }
    }

    /// Run the MakeMKV copy command to copy a disc's titles to the file system.
    ///
    /// **Note** This is called as part of the [`DriveActorHandle::copy_disc`] request.
    ///
    /// `command_output`:  Channel used by the MakeMKV command to relay output from the command as
    /// well as progress information.
    ///
    /// `device_path`:  Device path (or name) of the optical drive to perform the operation on
    /// (e.g. "/dev/sr0").
    ///
    /// `output_dir`:  The directory location where the video files should be written to.
    ///
    /// `log_file`:  The file location where the output of the command should be logged to.
    ///
    /// `cancellation_token`:  Cancellation token used to cancel the copy operation. It is assumed
    /// that the token is not already cancelled.
    ///
    /// # Errors
    ///
    /// [`Error::MakeMKV`] if an error occures while running the MakeMKV command.
    ///
    /// [`Error::DriveActorChannel`] if an error occures when attempting to send information thru
    /// a channel.
    pub async fn run_makemkv_copy(
        &self,
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: &str,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
    ) -> Result<CopyCommandOutput> {
        let (tx, rx) = oneshot::channel();

        let msg = DriveActorMessage::RunMakeMkvCopy {
            command_output,
            device_path: device_path.to_owned(),
            output_dir,
            log_file,
            cancellation_token,
            response: tx,
        };

        self.tx.send(msg).await
            .map_err(send_error)?;

        match rx.await {
            Ok(result) => result,
            Err(error) => Err(oneshot_recv_error(error)),
        }
    }

    /// Reset the drive state back to `Idle` from `Success` or `Failed`.
    pub async fn reset(&self) -> Result<()> {
        let msg = DriveActorMessage::Reset;
        self.tx.send(msg).await.map_err(send_error)
    }

    /// Updates the form data in the drive's persistent data.
    pub async fn update_form_data(&self, data: FormDataUpdate) -> Result<()> {
        let msg = DriveActorMessage::UpdateFormData { data };
        self.tx.send(msg).await.map_err(send_error)
    }

    /// Updates the current state of the optical drive.
    pub async fn update_optical_drive_state(&self, state: OpticalDriveState) -> Result<()> {
        let msg = DriveActorMessage::UpdateOpticalDriveState { state };
        self.tx.send(msg).await.map_err(send_error)
    }
}

/// Create an application error from the provided tokio error.
fn oneshot_recv_error(e: oneshot::error::RecvError) -> Error {
    Error::DriveActorChannel { error: Box::new(ChannelError::OneShotRecv(e)) }
}


/// Create an application error from the provided tokio error.
fn send_error(e: mpsc::error::SendError<DriveActorMessage>) -> Error {
    Error::DriveActorChannel { error: Box::new(ChannelError::Send(e)) }
}

