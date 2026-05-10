// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles drive operations on a worker node.
//!
//! This drive actor will run on worker nodes. It will monitor the state of the drive and send
//! updates to the control node. It is also responsible for performing MakeMKV operations.
//!
//! The drive actor can be initialized by calling [`init`]. This will start the task used by the
//! actor to process requests. Monitoring isn't handled directly by this task. It is instead
//! handled via the task created by the drive actor manager. The drive actor manager is responsible
//! for creating the drive actor instances.
//!
//! Requests made to the drive actor are typically done using the helper methods provided by the
//! [`crate::drive`] module.

use tokio_util::sync::CancellationToken;

use makemkv::{CopyCommandOutput, InfoCommandOutput};

use crate::{Error, Result};
use crate::actor::{self, Response};
use crate::bus;
use crate::drive::{
    self,
    DiscState,
    DriveRequest,
    Handle,
    Message,
    OsOpticalDrive
};
use crate::models::MediaLocation;
use crate::net;

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

    /// Cancellation token used to cancel a running MakeMKV command.
    cancellation_token: Option<CancellationToken>,
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
            cancellation_token: None,
        }
    }

    /// Cancels a running MakeMKV command.
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    fn makemkv_cancel(&mut self, resp: Response<()>) -> Result<()> {
        if let Some(ct) = self.cancellation_token.take() {
            ct.cancel();
        }

        tracing::info!(sn=self.drive.serial_number, "makemkv cancelled");

        resp.send(Ok(()))
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "WorkerMakeMkvCancel"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Send the current progress of the active MakeMKV command to the control node.
    ///
    /// # Args
    ///
    /// `op`:  Title of the current operation.
    ///
    /// `op_prog`:  Progress of the current operation.
    ///
    /// `subop`:  Title of the current suboperation.
    ///
    /// `subop_prog`:  Progress of the current suboperation.
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    async fn makemkv_progress(
        &self,
        op: String,
        op_prog: u8,
        subop: String,
        subop_prog: u8,
        resp: Response<()>
    ) -> Result<()> {
        let reply = if self.cancellation_token.is_some() {
            ignore_disconnected(
                net::send_makemkv_progress(
                    &self.bus,
                    &self.drive.serial_number,
                    op,
                    op_prog,
                    subop,
                    subop_prog
                ).await
            )
        } else {
            // In testing, it appeared that the complete notification would happen before all the
            // progress updates were processed. Ignore any progress updates that come in after the
            // command has stopped.
            Ok(())
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvProgress"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Send the output of a completed MakeMKV copy command to the control node.
    ///
    /// # Args
    ///
    /// `output`:  The output of the copy command.
    ///
    /// `resp`:  The transmission end of the channel to send the response to the request.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    async fn makemkv_copy_complete(
        &mut self,
        output: CopyCommandOutput,
        resp: Response<()>,
    ) -> Result<()> {
        self.cancellation_token = None;

        let reply = ignore_disconnected(
            net::send_makemkv_copy_complete(
                &self.bus,
                &self.drive.serial_number,
                output
            ).await
        );

        tracing::info!(sn=self.drive.serial_number, "makemkv copy complete");

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvCopyComplete"))
            .map_err(|_| Error::ResponseSend)
    }


    /// Send the output of a completed MakeMKV info command to the control node.
    ///
    /// # Args
    ///
    /// `output`:  The output of the info command.
    ///
    /// `resp`:  The transmission end of the channel to send the response to the request.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    async fn makemkv_info_complete(
        &mut self,
        output: InfoCommandOutput,
        resp: Response<()>,
    ) -> Result<()> {
        self.cancellation_token = None;

        let reply = ignore_disconnected(
            net::send_makemkv_info_complete(
                &self.bus,
                &self.drive.serial_number,
                output
            ).await
        );

        tracing::info!(sn=self.drive.serial_number, "makemkv info complete");

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvInfoComplete"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Send the error information for a failed MakeMKV command to the control node.
    ///
    /// # Args
    ///
    /// `output`:  The output of the copy command.
    ///
    /// `resp`:  The transmission end of the channel to send the response to the request.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    async fn makemkv_failed(&mut self, error: String, resp: Response<()>) -> Result<()> {
        let reply = if self.cancellation_token.is_some() {
            self.cancellation_token = None;
            tracing::info!(sn=self.drive.serial_number, ?error, "makemkv failed");
            ignore_disconnected(
                net::send_makemkv_failed(&self.bus, &self.drive.serial_number, error).await
            )
        } else {
            // MakeMKV was cancelled.
            Ok(())
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvFailed"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Runs the MakeMKV copy command to copy the titles on the disc to the file system.
    ///
    /// # Args
    ///
    /// `device`:  Device path (or name) of the optical drive to perform the copy operation on
    /// (e.g. "/dev/sr0").
    ///
    /// `output_dir`:  The directory location where the video files should be written to.
    ///
    /// `log_file`:  The file location where the output of the command should be logged to.
    ///
    /// `resp`:  The transmission end of the channel to send the response. The response will be
    /// sent once the command has been started.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidMediaLocation`] if one of the provided media locations is invalid.
    ///
    /// [`Error::MakeMkv`] if an error occures while running the MakeMKV command.
    fn run_makemkv_copy(
        &mut self,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        resp: Response<()>,
    ) -> Result<()> {
        if self.cancellation_token.is_some() {
            tracing::error!(sn=self.drive.serial_number, "MakeMKV command already running");
            return Err(Error::AlreadyRunning);
        }

        self.cancellation_token = Some(CancellationToken::new());

        let reply = drive::makemkv::run_makemkv_copy(
            &self.bus,
            &self.drive.serial_number,
            &self.drive.path,
            output_dir,
            log_file,
            self.cancellation_token.as_ref().unwrap().clone(),
        );

        tracing::info!(sn=self.drive.serial_number, "makemkv copy started");

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "WorkerRunMakeMkvCopy"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Runs the MakeMKV info command to gather information about the disc's titles.
    ///
    /// # Args
    ///
    /// `log_file`:  The file location where the output of the command should be logged to.
    ///
    /// `resp`:  The transmission end of the channel to send the response. The response will be
    /// sent once the command has been started.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidMediaLocation`] if the provided log file location isn't valid
    ///
    /// [`Error::MakeMkv`] if an error occures while running the MakeMKV command.
    fn run_makemkv_info(&mut self, log_file: MediaLocation, resp: Response<()>) -> Result<()> {
        if self.cancellation_token.is_some() {
            tracing::error!(sn=self.drive.serial_number, "MakeMKV command already running");
            return Err(Error::AlreadyRunning);
        }

        self.cancellation_token = Some(CancellationToken::new());

        let reply = drive::makemkv::run_makemkv_info(
            &self.bus,
            &self.drive.serial_number,
            &self.drive.path,
            log_file,
            self.cancellation_token.as_ref().unwrap().clone(),
        );

        tracing::info!(sn=self.drive.serial_number, "makemkv info started");

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "WorkerRunMakeMkvCopy"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Handler for a request not supported on the worker node.
    ///
    /// # Args
    ///
    /// `request`:  The name of the request.
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    fn unsupported_request<T>(&self, request: &str, resp: Response<T>) -> Result<()> {
        resp.send(Err(Error::UnsupportedRequest { request: request.to_owned() }))
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, request))
            .map_err(|_| Error::ResponseSend)
    }

    /// Send the updated drive information to the control node.
    ///
    /// # Args
    ///
    /// `drive`:  The updated drive information. If `None`, then it is assumed that the drive has
    /// become disconnected.
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    async fn update_from_os(&mut self, drive: OsOpticalDrive, resp: Response<()>) -> Result<()> {
        // TODO: Should only send a message if something changed or if this is the first update
        //       since the control node connected. Could also skip if drive is performing a copy
        //       operation.

        self.drive = drive.clone();
        let reply = ignore_disconnected(net::send_drive_status_update(&self.bus, drive).await);

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "UpdateFromOs"))
            .map_err(|_| Error::ResponseSend)
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        let request = msg.drive_request(&self.drive.serial_number)?;

        match request {
            DriveRequest::BeginCopyDisc { params: _, response } => {
                self.unsupported_request("BeginCopyDisc", response)
            },
            DriveRequest::CancelCopyDisc { response } => {
                self.unsupported_request("CancelCopyDisc", response)
            },
            DriveRequest::CopyCompleted { response } => {
                self.unsupported_request("CopyCompleted", response)
            },
            DriveRequest::CopyFailed { error: _, response } => {
                self.unsupported_request("CopyFailed", response)
            },
            DriveRequest::CheckDriveStatus { response } => {
                self.unsupported_request("CheckDriveStatus", response)
            },
            DriveRequest::GetStatus { response } => {
                self.unsupported_request("GetStatus", response)
            },
            DriveRequest::MakeMkvCopyComplete { output, response } => {
                self.makemkv_copy_complete(output, response).await
            },
            DriveRequest::MakeMkvFailed { error, response } => {
                self.makemkv_failed(error, response).await
            },
            DriveRequest::MakeMkvInfoComplete { output, response } => {
                self.makemkv_info_complete(output, response).await
            },
            DriveRequest::MakeMkvProgress { op, op_prog, subop, subop_prog, response } => {
                self.makemkv_progress(op, op_prog, subop, subop_prog, response).await
            },
            DriveRequest::ReadFormData { response } => {
                self.unsupported_request("ReadFormData", response)
            },
            DriveRequest::Reset { response } => {
                self.unsupported_request("Reset", response)
            },
            DriveRequest::RunMakeMkvCopy {
                output_dir: _,
                log_file: _,
                cancellation_token: _,
                response,
            } => {
                self.unsupported_request("RunMakeMkvCopy", response)
            },
            DriveRequest::RunMakeMkvInfo {
                log_file: _,
                cancellation_token: _,
                response,
            } => {
                self.unsupported_request("RunMakeMkvInfo", response)
            },
            DriveRequest::SaveFormData { data: _, response } => {
                self.unsupported_request("SaveFormData", response)
            },
            DriveRequest::UpdateFromOs { drive, response, worker: _ } => {
                self.update_from_os(drive, response).await
            },
            DriveRequest::WorkerMakeMkvCancel { response } => {
                self.makemkv_cancel(response)
            },
            DriveRequest::WorkerRunMakeMkvCopy { output_dir, log_file, response } => {
                self.run_makemkv_copy(output_dir, log_file, response)
            },
            DriveRequest::WorkerRunMakeMkvInfo { log_file, response } => {
                self.run_makemkv_info(log_file, response)
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

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
