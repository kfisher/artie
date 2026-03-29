// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::Result;
use crate::error::{ChannelError, Error};
use crate::drive::{OpticalDriveState, OpticalDriveStatus};
use crate::drive::actor::DriveActorMessage;
use crate::drive::data::{FormData, FormDataUpdate};
use crate::models::CopyParamaters;

/// Handle used to communicate with a [`DriveActor`] instance.
// TODO: Update
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

