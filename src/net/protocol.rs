// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the messages sent between the control and worker nodes.

use serde::{self, Deserialize, Serialize};
use serde_json;

use makemkv::{CopyCommandOutput, InfoCommandOutput};

use crate::Result;
use crate::drive::OsOpticalDrive;
use crate::models::MediaLocation;
use crate::net::{self, IncomingMessage};

/// Messages that can be send between the control and worker nodes.
///
/// For each variant, the documentation contains one of the following notations:
///
/// - (c -> w): Indicates the message is meant to be sent from the control node to a worker node.
/// - (w -> c): Indicates the message is meant to be sent from a worker node to the control node.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    /// (w -> c) The updated status of an optical drive.
    DriveStatusUpdate {
        drive: OsOpticalDrive,
    },

    /// (c -> w) Request to run the MakeMKV copy command.
    /// system.
    RunMakeMkvCopy {
        drive: String,
        output_dir: MediaLocation,
        log_file: MediaLocation,
    },

    /// (c -> w) Request to run the MakeMKV info command.
    RunMakeMkvInfo {
        drive: String,
        log_file: MediaLocation,
    },

    /// (c -> w) Cancel an in-progress MakeMKV operation (copy or info).
    MakeMkvCancel {
        drive: String,
    },

    /// (w -> c) MakeMKV command failed.
    MakeMkvFailed {
        drive: String,
        error: String,
    },

    /// (w -> c) The result of running a successfull MakeMKV copy command.
    MakeMkvCopyComplete {
        drive: String,
        output: CopyCommandOutput,
    },

    /// (w -> c) The result of running a successfull MakeMKV info command.
    MakeMkvInfoComplete {
        drive: String,
        output: InfoCommandOutput,
    },

    /// (w -> c) Progress information about a running MakeMKV command.
    MakeMkvProgress {
        drive: String,
        op: String,
        op_prog: u8,
        subop: String,
        subop_prog: u8,
    },
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
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Convert the network message into a incoming message for the network actor.
    ///
    /// # Args
    ///
    /// `sender`:  The IP address of the
    pub fn incoming_message(self, sender: &str) -> net::Message {
        let msg = IncomingMessage {
            msg: self,
            sender: sender.to_owned(),
        };
        net::Message::Incoming(msg)
    }
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
