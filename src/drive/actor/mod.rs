// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

pub mod handle;
pub mod local;

use tokio::sync::oneshot;

use crate::drive::{OpticalDriveState, OpticalDriveStatus};
use crate::drive::data::{FormData, FormDataUpdate};
use crate::models::CopyParamaters;

/// Specifies the messages and responses for the optical drive actor.
///
/// Each message will have a channel that will be used by the actor to send the response. This will
/// be a `oneshot` channel for a single response or a `mpsc` channel for streaming data such as
/// operation process information.
#[derive(Debug)]
pub enum DriveActorMessage {
    /// Request to cancel and in-progress copy operation.
    CancelCopyDisc,

    /// Request to start copying the disc in the optical drive.
    CopyDisc {
        parameters: CopyParamaters,
    },

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
