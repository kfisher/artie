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
    // TODO: This is a temporary hack for testing multi-node on a single system. 
    let drives = if mode == Mode::Worker {
        drive::get_optical_drives()?
    } else {
        Vec::new()
    };

    for drive in &drives {
        let serial_number = drive.serial_number.clone();
        task::spawn(monitor::monitor_drive(bus.clone(), serial_number));
    }

    let msg_processor = MessageProcessor::new(bus.clone(), mode);
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

    /// The mode the application is running in.
    mode: Mode,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    ///
    /// # Args
    ///
    /// `bus`:  Handle used to send messages to other actors via the message bus.
    ///
    /// `mode`:  The mode the application is running in.
    fn new(bus: bus::Handle, mode: Mode) -> Self {
        Self { bus, drives: Vec::new(), mode }
    }

    /// Gets the handle for optical drive actor.
    ///
    /// If a handle does not exist, a new actor instance will be created for it. The type of actor
    /// will depend on the mode the application instance is running in.
    ///
    /// # Args
    ///
    /// `serial_number`:  The serial number of the optical drive whose actor handle should be
    /// returned.
    fn get_or_add_drive(&mut self, serial_number: &str) -> &DriveHandle {
        if let Some(pos) = self.drives.iter().position(|d| d.serial_number == serial_number) {
            return &self.drives[pos];
        }

        let drive = match self.mode {
            Mode::Control => {
                drive::actor::init(self.bus.clone(), serial_number)
            },
            Mode::Worker => {
                drive::worker::init(self.bus.clone(), serial_number)
            },
        };

        let drive = DriveHandle {
            serial_number: serial_number.to_owned(),
            actor: drive,
        };

        self.drives.push(drive);

        // Since we just added an item to the vector, its should be safe to unwrap.
        self.drives.last().unwrap()
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
                let drive = self.get_or_add_drive(serial_number);
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

