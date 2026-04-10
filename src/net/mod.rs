// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles network communication.

pub mod client;
pub mod server;
pub mod messages;

use serde::{Deserialize, Serialize};

/// Networking application settings.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    /// List of IP addresses for worker nodes.
    ///
    /// Only valid for the control node application instance.
    pub workers: Vec<String>,
}
