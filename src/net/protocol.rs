// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the messages sent between the control and worker nodes.

use serde::{self, Deserialize, Serialize};
use serde_json;

use crate::Result;
use crate::drive::OsOpticalDrive;
use crate::net::{self, IncomingMessage};

/// Messages that can be send between the control and worker nodes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    /// (w -> c) The updated status of an optical drive.
    ///
    /// If `drive` is `None`, then the drive information was no longer being reported by the OS
    /// meaning it was disconnected.
    DriveStatusUpdate {
        drive: Option<OsOpticalDrive>,
    }
}

impl Message {
    /// Parse a message received from the network.
    ///
    /// # Args
    ///
    /// `bytes`:  The raw message. It is expected to be the JSON representation of the message
    /// using the [adjacently tagged](https://serde.rs/enum-representations.html#adjacently-tagged)
    /// enum representation method.
    ///
    /// # Errors
    ///
    /// [`crate::Error::SerdeJson`] if the bytes cannot be deserialized.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }

    /// Serializes the message as JSON.
    ///
    /// This will automatically add a newline character to the end of the byte array before
    /// returning.
    ///
    /// # Errors
    ///
    /// [`crate::Error::SerdeJson`] if the bytes cannot be serialized.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes = serde_json::to_vec(self)?;
        bytes.push('\n' as u8);
        Ok(bytes)
    }

    /// Convert the network message into a incoming message for the network actor.
    pub fn to_incoming_message(self) -> net::Message {
        let msg = IncomingMessage {
            msg: self,
        };
        net::Message::Incoming(msg)
    }
}

