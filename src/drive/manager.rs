// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Actor for managing drive actors.
//!
//! The drive manager actor is responsible for managing the other drive actor instances. It also
//! serves as the broker for drive related requests coming from the message bus.

use crate::{Error, Mode, Result};
use crate::actor::{self, Response};
use crate::bus;
use crate::drive::{self, Handle, Message};
use crate::drive::monitor;
use crate::task;

/// Optical drive manager requests
#[derive(Debug)]
pub enum ManagerRequest {
    /// Get list of drive serial numbers.
    ///
    /// This will return the serial numbers for all optical drives that have an associated drive
    /// actor. This includes local drives and those on remote worker nodes. Use the appropriate
    /// drive specific request to get details about the drives.
    GetDrives {
        response: Response<Vec<String>>,
    },
}

/// Create the drive actor manager and spawn the task used to process its requests.
///
/// # Args
///
/// `bus`:  Handle used to send messages to other actors via the message bus.
///
/// `mode`:  The mode the application is running as. This will control what type of actor is
/// created for each locally connected drives.
///
/// # Errors
///
/// Will return errors if the command to get optical drive data from the OS fails. This will vary
/// based on OS type.
pub fn init(bus: &bus::Handle, mode: Mode) -> Result<Handle> {
    // TODO: Temporary
    let drives = if mode == Mode::Worker {
        drive::get_optical_drives()?
    } else {
        Vec::new()
    };

    let drives: Vec<DriveHandle> = drives.into_iter()
        .map(|drive| {
            let serial_number = drive.serial_number.clone();
            let handle = if mode == Mode::Control {
                drive::actor::init(bus.clone(), drive)
            } else {
                drive::worker::init(bus.clone(), drive)
            };

            DriveHandle { serial_number, actor: handle }
        })
        .collect();

    for drive in &drives {
        let serial_number = drive.serial_number.clone();
        task::spawn(monitor::monitor_drive(bus.clone(), serial_number));
    }

    let msg_processor = MessageProcessor::new(bus.clone(), drives);
    Ok(actor::create_and_run("drive manager", msg_processor))
}

/// Handle for interfacing with a drive actor.
///
/// This is essentially the same as the standard handle type, but with the addition of the serial
/// number of the associated drive.
struct DriveHandle {
    /// The serial number of the optical drive.
    serial_number: String,

    /// The underlying actor handle.
    actor: Handle,
}

/// Processes messages sent to the actor manager.
///
/// The actor manager will receive both messages meant for it specifically as well as the messages
/// intended for drive actors which it will forward on.
struct MessageProcessor {
    /// Handle used to send messages to other actors via the message bus.
    bus: bus::Handle,

    /// List of drive actor handles for all available optical drives.
    ///
    /// If the application instance is the control node, this can contain both local actors and
    /// worker client actors. If this is a worker node, this will only contain worker actors.
    drives: Vec<DriveHandle>,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    ///
    /// # Args
    ///
    /// `bus`:  Handle used to send messages to other actors via the message bus.
    ///
    /// `drives`:  Initial list of optical drives.
    fn new(bus: bus::Handle, drives: Vec<DriveHandle>) -> Self {
        // let drives = drives.into_iter()
        //     // create_local_actor will log and error message if the actor could not be created for
        //     // some reason.
        //     .filter_map(|drive| create_local_actor(&bus, drive).ok())
        //     .collect();

        Self { bus, drives }
    }

    /// Get list of drive serial numbers.
    /// 
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::get_drives`] for more information on the response, including potential errors that
    /// could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    async fn get_drives(&self, resp: Response<Vec<String>>) -> Result<()> {
        let serial_numbers = self.drives.iter()
            .map(|drive| drive.serial_number.clone())
            .collect();
        resp.send(Ok(serial_numbers))
            .inspect_err(|_| send_error_trace("GetDrives"))
            .map_err(|_| Error::ResponseSend)
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Drive { ref serial_number, request: _ } => {
                let drive = self.drives.iter()
                    .find(|drive| drive.serial_number == *serial_number)
                    .ok_or(Error::DriveNotFound { serial_number: serial_number.clone() })?;
                drive.actor.send(msg).await
            },
            Message::Manager { request } => {
                match request {
                    ManagerRequest::GetDrives { response } => {
                        self.get_drives(response).await
                    },
                }
            },
        }
    }
}

/// Log an error due to failure to send a response.
/// 
/// # Args
///
/// `request`:  The name of the request the response was being sent for.
fn send_error_trace(request: &str) {
    tracing::error!("failed to send {} response", request);
}

#[cfg(test)]
mod tests {
    // TODO
}

