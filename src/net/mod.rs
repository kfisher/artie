// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles network communication between the control and worker nodes.
//!
//! In multi-node configurations, there will be one control node and one or more worker nodes. The
//! control node will be the node the user interacts with and will be responsible for managing the
//! application data (e.g. database).
//!
//! Worker nodes will wait for commands from the control node. The primary purpose of worker nodes
//! will be to perform copy and transcode operations.
//!
//! Worker nodes will operate as a server (see [`server`]) and the control node will operate as a
//! client connecting to the workers (see [`client`]).

pub mod client;
pub mod server;

use serde::{Deserialize, Serialize};

use crate::actor;

/// Handle used to communicate with the client or server actor.
pub type Handle = actor::Handle<Message>;

/// Message for sending requests to the client or server actor.
#[derive(Debug)]
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
