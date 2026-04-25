// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles network communication between the control and worker nodes.

mod server;

use serde::{Deserialize, Serialize};

use crate::actor;

/// Handle used to communicate with the client or server actor.
pub type Handle = actor::Handle<Message>;

/// Message for sending requests to the client or server actor.
pub enum Message {
}

/// Networking application settings.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    /// List of IP addresses for worker nodes.
    ///
    /// Only valid for the control node application instance.
    pub workers: Vec<String>,
}
