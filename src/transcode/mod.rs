// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles all transcode operations.
//!
//! TODO

mod actor;
mod handbrake;
mod manager;
mod worker;

pub use actor::TranscodeRequest;
pub use manager::{ManagerRequest, init};

/// Handle used to communicate with the transcode actors and managers.
pub type Handle = crate::actor::Handle<Message>;

/// Message for sending requests to a transcode actor or the transcode manager.
#[derive(Debug)]
pub enum Message {
    /// Message type for sending requests to a transcode actor.
    ///
    /// Transcode actor instances are identified using their `hostname` since each host can only
    /// perform one transcode operation at a time.
    Actor {
        hostname: String,
        request: TranscodeRequest,
    },

    /// Message type for sending requests to the transcode manager, 
    Manager {
        request: ManagerRequest,
    },
}

impl Message {
}

