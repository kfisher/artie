// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Messages sent between the control and worker nodes.

use serde::{Deserialize, Serialize};

use crate::{Error, Result};
use crate::error;
use crate::drive::OpticalDriveStatus;

/// Specifies the different message types.
#[derive(Debug)]
pub enum Message {
    /// Message containing the current status of an optical drive.
    DriveStatus(OpticalDriveStatus),
}

impl Message {
    /// Deserializes a message from the network.
    pub fn deserialize<T>(lines: &mut T) -> Result<Message> 
    where
        T: Iterator<Item = String>
    {
        let msg_type = lines
            .next()
            .ok_or_else(|| Error::MessageParseError {
                error: String::from("message was empty") 
            })?;
        let msg_data = lines
            .next()
            .ok_or_else(|| Error::MessageParseError {
                error: String::from("message missing data") 
            })?;

        match msg_type.as_str() {
            "DriveStatus" => {
                let drive_status = serde_json::from_str::<OpticalDriveStatus>(&msg_data)
                    .map_err(|e| error::json_deserialize(e))?;
                Ok(Message::DriveStatus(drive_status))
            },
            _ => {
                Err(Error::InvalidMessageType { msg_type, msg_data })
            }
        }

    }

    /// Serializes the message for transmission over the network.
    pub fn serialize(&self) -> Result<String> {
        match self {
            Message::DriveStatus(drive_status) => {
                let msg_type = "DriveStatus";
                let msg_data = serde_json::to_string(drive_status)
                    .map_err(|e| error::json_serialize(e))?;
                Ok(format_message(msg_type, &msg_data))
            },
        }
    }

}

/// Formats a message for transmission.
fn format_message(msg_type: &str, msg_data: &str) -> String {
    format!("{}\n{}\n\n", msg_type, msg_data)
}

// TODO: Testing
