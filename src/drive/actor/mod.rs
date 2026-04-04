// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

pub mod handle;
pub mod local;
pub mod worker;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use tokio_util::sync::CancellationToken;

use makemkv::{CommandOutput, CopyCommandOutput, InfoCommandOutput};

use crate::Result;
use crate::drive::{OpticalDriveState, OpticalDriveStatus};
use crate::drive::data::{FormData, FormDataUpdate};
use crate::models::{CopyParamaters, MediaLocation};

/// Maxium number of messages for a drive actor that can be queued.
const ACTOR_CHANNEL_BUFFER_SIZE: usize = 10;

/// Specifies the messages and responses for the optical drive actor.
#[derive(Debug)]
pub enum DriveActorMessage {
    /// Request to cancel and in-progress copy operation.
    CancelCopyDisc,

    /// Request to start copying the disc in the optical drive.
    CopyDisc {
        parameters: CopyParamaters,
    },

    // TODO: Should the responses send a result?

    /// Request the form data from the drive's persistent data.
    GetFormData {
        response: oneshot::Sender<FormData>,
    },

    /// Request the current status of the optical drive.
    GetStatus {
        response: oneshot::Sender<OpticalDriveStatus>,
    },

    /// Request to reset the drive state back to idle.
    ///
    /// Resets the state from `Success` or `Failed` back to `Idle`.
    Reset,

    /// Request to run the MakeMKV info command to gather information about the titles on the disc.
    RunMakeMkvInfo {
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: String,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: oneshot::Sender<Result<InfoCommandOutput>>,
    },

    /// Request to run the MakeMKV copy command to copy titles from the disc to the file system.
    RunMakeMkvCopy {
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: String,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: oneshot::Sender<Result<CopyCommandOutput>>,
    },

    /// Update the form data stored in the drive's persistent data.
    ///
    /// Each field is optional that are `Some` if the field was modified or `None` if the field
    /// hasn't changed.
    UpdateFormData {
        data: FormDataUpdate,
    },

    /// Updates the optical drive state.
    UpdateOpticalDriveState {
        state: OpticalDriveState,
    }
}

/// Handles copy operations and monitoring the state of an optical drive.
pub trait DriveActor {
    /// Process a message sent to the actor.
    fn proc_msg(&mut self, msg: DriveActorMessage) -> Result<()>;

    /// Get the serial number of the drive associated with the actor.
    fn serial_number(&self) -> &str;

    /// Get the next message in the queue.
    ///
    /// This will return `None` when the message channel is closed and does not contain any queued
    /// messages. If the message queue is empty, but the channel is not closed, this will sleep
    /// until a message is sent or the channel is closed.
    async fn recv_msg(&mut self) -> Option<DriveActorMessage>;
}

/// Runs the processing loop for the provided actor.
///
/// This processing loop will run until the actor's message channel is closed.
pub async fn run_actor<T: DriveActor>(mut actor: T) {
    tracing::info!(sn=actor.serial_number(), "message processing started");

    while let Some(msg) = actor.recv_msg().await {
        if let Err(error) = actor.proc_msg(msg) {
            tracing::error!(sn=actor.serial_number(), ?error, "failed to process message.");
        }
    }

    tracing::info!(sn=actor.serial_number(), "message processing ended");
}

