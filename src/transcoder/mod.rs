// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles all transcode operations.
//!
//! TODO

use tokio::sync::oneshot;

mod actor;
mod handbrake;
mod manager;
mod worker;

use crate::Result;
use crate::bus;

pub use actor::TranscoderRequest;
pub use manager::{ManagerRequest, init};

/// Handle used to communicate with the transcoder actors and managers.
pub type Handle = crate::actor::Handle<Message>;

/// Message for sending requests to a transcodes actor or the transcoder manager.
#[derive(Debug)]
pub enum Message {
    /// Message type for sending requests to the transcoder manager, 
    Manager {
        request: ManagerRequest,
    },

    /// Message type for sending requests to a transcoder actor.
    Transcoder {
        id: String,
        request: TranscoderRequest,
    },
}

impl Message {
}

/// TODO
#[derive(Debug)]
pub struct Transcoder {
}

/// Get the current status of a transcoder.
pub async fn get(bus: &bus::Handle, transcoder_id: &str) -> Result<Transcoder> {
    let (tx, rx) = oneshot::channel();
    let request = TranscoderRequest::GetStatus { response: tx };
    let msg = Message::Transcoder { id: transcoder_id.to_owned(), request };
    bus.send(msg).await?;
    rx.await?
}

/// Get list of the identifiers for all known transcoder instances.
pub async fn list(bus: &bus::Handle) -> Result<Vec<String>> {
    let (tx, rx) = oneshot::channel();
    let request = ManagerRequest::GetTranscoders { response: tx };
    let msg = Message::Manager { request };
    bus.send(msg).await?;
    rx.await?
}
