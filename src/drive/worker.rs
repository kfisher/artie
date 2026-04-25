// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use crate::Result;
use crate::actor;
use crate::drive::{DriveRequest, Handle, Message, OsOpticalDrive};

// TODO
pub fn init(drive: OsOpticalDrive) -> Handle {
    let msg_processor = MessageProcessor::new(drive);
    let name = format!("drive {}", &msg_processor.drive.serial_number);
    actor::create_and_run(&name, msg_processor)
}

/// Processes messages sent to the control drive actor.
struct MessageProcessor {
    // TODO
    drive: OsOpticalDrive,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    ///
    /// # Args
    ///
    /// `drive`:  TODO
    fn new(drive: OsOpticalDrive) -> Self {
        Self { drive }
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        let request = msg.drive_request(&self.drive.serial_number)?;

        match request {
            DriveRequest::BeginCopyDisc { params: _, response: _ } => {
                // Not supported.
                todo!()
            },
            DriveRequest::CancelCopyDisc { response: _ } => {
                // ?
                todo!()
            },
            DriveRequest::GetStatus { response: _ } => {
                // Not supported.
                todo!()
            },
            DriveRequest::ReadFormData { response: _ } => {
                // Not supported.
                todo!()
            },
            DriveRequest::Reset { response: _ } => {
                // Not supported.
                todo!()
            },
            DriveRequest::RunMakeMkvCopy {
                command_output: _,
                device_path: _,
                output_dir: _,
                log_file: _,
                cancellation_token: _,
                response: _,
            } => {
                // Same as local.
                todo!()
            },
            DriveRequest::RunMakeMkvInfo {
                command_output: _,
                device_path: _,
                log_file: _,
                cancellation_token: _,
                response: _,
            } => {
                // Same as local.
                todo!()
            },
            DriveRequest::SaveFormData { data: _, response: _ } => {
                // Not supported.
                todo!()
            },
            DriveRequest::UpdateFromCopy { state: _, response: _ } => {
                // Send update to the control node.
                todo!()
            },
            DriveRequest::UpdateFromOs { info: _, response } => {
                // Send update to the control node.
                tracing::info!("UpdateFromOs");
                let _ = response.send(Ok(()));
                Ok(())
            },
        }
    }
}

