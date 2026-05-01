// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use crate::{Error, Result};
use crate::actor::{self, Response};
use crate::bus;
use crate::drive::{DiscState, DriveRequest, Handle, Message, OsOpticalDrive};
use crate::net;
use crate::task;

/// Create the worker actor instance for the provided drive.
///
/// `bus`:  Handle used to send messages to other actors via the message bus.
///
/// `serial_number`:  The serial number of the drive the actor is being created for.
pub fn init(bus: bus::Handle, serial_number: &str) -> Handle {
    let msg_processor = MessageProcessor::new(bus, serial_number);
    let name = format!("drive {}", &serial_number);
    actor::create_and_run(&name, msg_processor)
}

/// Processes messages sent to the control drive actor.
struct MessageProcessor {
    /// The optical drive associated with the actor instance that this message processor will be
    /// processing messages for.
    drive: OsOpticalDrive,

    /// Handle used to send messages to other actors via the message bus.
    bus: bus::Handle,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    ///
    /// # Args
    ///
    /// `bus`:  Handle used to send messages to other actors via the message bus.
    ///
    /// `serial_number`:  The serial number of the optical drive associated with the actor instance
    /// that this message processor will be processing messages for.
    fn new(bus: bus::Handle, serial_number: &str) -> Self {
        Self {
            bus,
            drive: OsOpticalDrive {
                path: String::default(),
                serial_number: serial_number.to_owned(),
                disc: DiscState::None,
                hostname: String::default(),
            },
        }
    }

    /// Send the updated drive information to the control node.
    ///
    /// # Args
    ///
    /// `drive`:  The updated drive information. If `None`, then it is assumed that the drive has
    /// become disconnected.
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    fn update_from_os(&mut self, drive: Option<OsOpticalDrive>, resp: Response<()>) -> Result<()> {
        // TODO: Should be updating the internal state as well.
        //
        // TODO: Should only send a message if something changed or if this is the first update
        //       since the control node connected. Could also skip if drive is performing a copy
        //       operation.
        let bus = self.bus.clone();
        let serial_number = self.drive.serial_number.clone();
        task::spawn(async move {
            let reply = ignore_disconnected(net::send_drive_status_update(&bus, drive).await);
            resp.send(reply)
                .inspect_err(|_| send_error_trace(&serial_number, "UpdateFromOs"))
                .map_err(|_| Error::ResponseSend)
        });

        Ok(())
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
            DriveRequest::UpdateFromOs { info, response } => {
                self.update_from_os(info, response)
            },
        }
    }
}

/// Change `Err` to `Ok` if the error is [`Error::Disconnected`].
fn ignore_disconnected(result: Result<()>) -> Result<()> {
    let Err(error) = result else {
        return Ok(());
    };

    match error {
        Error::Disconnected => Ok(()),
        _ => Err(error)
    }
}


/// Log an error due to failure to send a response.
/// 
/// # Args
///
/// `serial_number`:  The serial number of the drive the request was for.
///
/// `request`:  The name of the request the response was being sent for.
fn send_error_trace(serial_number: &str, request: &str) {
    tracing::error!(sn=serial_number, "failed to send {} response", request);
}
