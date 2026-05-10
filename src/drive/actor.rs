// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles drive operations on the control node.
//!
//! This drive actor runs on the control node. For drives attached to the same host that the
//! control node is run on, it will handle monitoring the drive state and running MakeMKV commands.
//! For drives not attached to the same host, it rely on a drive actor running on a worker node to
//! handle these operations.
//!
//! All other drive operations are handled exclusively handle by this drive actor.
//!
//! The drive actor can be initialized by calling [`init`]. This will start the task used by the
//! actor to process requests. Monitoring isn't handled directly by this task. It is instead
//! handled via the task created by the drive actor manager. See the [`crate::drive::manager`] and
//! [`crate::drive::monitor`] modules.
//!
//! Requests made to the drive actor are typically done using the helper methods provided by the
//! [`crate::drive`] module.

use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;

use makemkv::{CopyCommandOutput, InfoCommandOutput};

use crate::{Error, Result};
use crate::actor::{self, Response};
use crate::bus;
use crate::drive::{
    self,
    FormData,
    FormDataUpdate,
    Handle,
    Message,
    OpticalDrive,
    OpticalDriveState,
    OsOpticalDrive,
};
use crate::drive::copy;
use crate::drive::data;
use crate::models::{CopyParamaters, MediaLocation};
use crate::net;
use crate::task;

/// Amount of time with getting a status update when the drive should be considered disconnected.
const DRIVE_TIMEOUT: Duration = Duration::from_secs(3);

/// Optical drive actor requests.
#[derive(Debug)]
pub enum DriveRequest {
    /// Begin copying a disc.
    BeginCopyDisc {
        params: CopyParamaters,
        response: Response<()>,
    },

    /// Cancel an in-progress copy operation.
    CancelCopyDisc {
        response: Response<()>,
    },

    /// Check the drive for stale status information.
    ///
    /// If a drive hasn't received an update in a while, its status will be marked as disconnected.
    CheckDriveStatus {
        response: Response<()>,
    },

    /// Notify the drive actor that the copy operation has failed or was cancelled.
    CopyFailed {
        error: String,
        response: Response<()>,
    },

    /// Notify the drive actor that the copy operation completed successfully.
    CopyCompleted {
        response: Response<()>,
    },

    /// Get the current status of an optical drive.
    GetStatus {
        response: Response<OpticalDrive>,
    },

    /// Notify the drive actor that the MakeMKV copy command completed successfully.
    MakeMkvCopyComplete {
        output: CopyCommandOutput,
        response: Response<()>,
    },

    /// Notify the drive actor that a MakeMKV command failed.
    MakeMkvFailed {
        error: String,
        response: Response<()>,
    },

    /// Notify the drive actor that the MakeMKV info command completed successfully.
    MakeMkvInfoComplete {
        output: InfoCommandOutput,
        response: Response<()>,
    },

    /// Progress information about a running MakeMKV command.
    MakeMkvProgress {
        op: String,
        op_prog: u8,
        subop: String,
        subop_prog: u8,
        response: Response<()>,
    },

    /// Get the last saved values for a drive's copy parameters.
    ReadFormData {
        response: Response<FormData>,
    },

    /// Reset the drive state back to idle.
    ///
    /// Should only be requested if the drive state is currently in `Success` or `Failed`. Will
    /// result in an error if in any other state.
    Reset {
        response: Response<()>,
    },

    /// Request to run the MakeMKV copy command to copy titles from the disc to the file system.
    RunMakeMkvCopy {
        output_dir: MediaLocation,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: Response<CopyCommandOutput>,
    },

    /// Request to run the MakeMKV info command to gather information about the titles on the disc.
    RunMakeMkvInfo {
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: Response<InfoCommandOutput>,
    },

    /// Update the copy parameters stored in the drive's persistent data.
    SaveFormData {
        data: FormDataUpdate,
        response: Response<()>,
    },

    /// Updates the current state of the drive.
    ///
    /// `worker` will only be `Some` on the control node for updates it gets from a worker node.
    UpdateFromOs {
        drive: OsOpticalDrive,
        worker: Option<String>,
        response: Response<()>,
    },

    /// Request to cancel a running MakeMKV operation.
    ///
    /// This is only applicable on the worker node and is used to send the request from the network
    /// actor to the worker drive actor.
    WorkerMakeMkvCancel {
        response: Response<()>,
    },

    /// Request to run the MakeMKV copy command to copy titles from the disc to the file system.
    ///
    /// This is only applicable on the worker node and is used to send the request from the network
    /// actor to the worker drive actor.
    WorkerRunMakeMkvCopy {
        output_dir: MediaLocation,
        log_file: MediaLocation,
        response: Response<()>,
    },

    /// Request to run the MakeMKV info command to gather information about the titles on the disc.
    ///
    /// This is only applicable on the worker node and is used to send the request from the network
    /// actor to the worker drive actor.
    WorkerRunMakeMkvInfo {
        log_file: MediaLocation,
        response: Response<()>,
    },
}

/// Create the drive actor instance for the provided drive.
///
/// This will create the actor and spawn the task for processing its requests.
///
/// # Args
///
/// `bus`:  Handle used to send messages to other actors via the message bus.
///
/// `serial_number`:  The serial number of the drive the actor is being created for.
pub fn init(bus: bus::Handle, serial_number: &str) -> Handle {
    let msg_processor = MessageProcessor::new(bus.clone(), serial_number);
    let name = format!("drive {}", &serial_number);
    actor::create_and_run(&name, msg_processor)
}

/// Processes messages sent to the drive actor.
struct MessageProcessor {
    /// Handle used to send messages to other actors via the message bus.
    bus: bus::Handle,

    /// The drive's current state.
    drive: OpticalDrive,

    /// Address of the worker node managing the drive.
    ///
    /// If `None`, the drive is managed by the same application instance this processor is running
    /// on which should be the control node.
    worker: Option<String>,

    /// Cancellation token used to cancel a copy operation.
    copy_ct: Option<CancellationToken>,

    /// The instant the currently running copy operation was started.
    copy_started: Option<Instant>,

    /// The last time a status update was received from the optical drive monitor task or from a
    /// running copy operation.
    last_update: Instant,

    /// The transmission end of the channel to send the result of a MakeMKV info command.
    makemkv_info_resp: Option<Response<InfoCommandOutput>>,

    /// The transmission end of the channel to send the result of a MakeMKV copy command.
    makemkv_copy_resp: Option<Response<CopyCommandOutput>>,
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
        let name = data::get_drive_name(&serial_number)
            .inspect_err(|error| {
                tracing::warn!(sn=serial_number, ?error, "failed to lookup drive name");
            })
            .unwrap_or(serial_number.to_owned());
        Self {
            bus,
            drive: OpticalDrive::disconnected(&name, serial_number),
            worker: None,
            copy_ct: None,
            copy_started: None,
            last_update: Instant::now(),
            makemkv_info_resp: None,
            makemkv_copy_resp: None,
        }
    }

    /// Begin copying a disc.
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::begin_copy`] for more information on the response, including potential errors that
    /// could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn begin_copy_disc(&mut self, params: CopyParamaters, resp: Response<()>) -> Result<()> {
        let reply = if self.drive.state == OpticalDriveState::Idle {
            self.drive.state = OpticalDriveState::Copying {
                stage: "",
                task: String::from(""),
                task_progress: 0.0,
                subtask: String::from(""),
                subtask_progress: 0.0,
                elapsed_time: Duration::ZERO,
            };

            self.copy_ct = Some(CancellationToken::new());
            self.copy_started = Some(Instant::now());

            let bus = self.bus.clone();
            let drive = self.drive.os_drive();
            let ct = self.copy_ct.as_ref().unwrap().clone();
            task::spawn(async move {
                copy::copy_disc(
                    bus,
                    drive,
                    params,
                    ct,
                ).await
            });

            Ok(())
        } else {
            Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "BeginCopy"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Cancel an in-progress copy operation.
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::cancel_copy`] for more information on the response, including potential errors
    /// that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    async fn cancel_copy_disc(&mut self, resp: Response<()>) -> Result<()> {
        if !self.drive.state.is_copying() {
            let error = Error::InvalidDriveState { state: self.drive.state.name().to_owned() };
            return resp.send(Err(error))
                .inspect_err(|_| send_error_trace(&self.drive.serial_number, "CancelCopyDisc"))
                .map_err(|_| Error::ResponseSend);
        }

        let Some(copy_ct) = self.copy_ct.take() else {
            let error = Error::CancelTokenNone;
            return resp.send(Err(error))
                .inspect_err(|_| send_error_trace(&self.drive.serial_number, "CancelCopyDisc"))
                .map_err(|_| Error::ResponseSend);
        };

        copy_ct.cancel();
        self.makemkv_info_resp = None;
        self.makemkv_copy_resp = None;

        if let Some(worker) = &self.worker {
            // On the off chance that the cancelled happened in-between MakeMKV commands, the
            // worker getting a cancel request when not running a command will be handled
            // gracefully.
            let _ = net::send_cancel_makemkv_op(&self.bus, worker, &self.drive.serial_number).await
                .inspect_err(|error| {
                    tracing::warn!(
                        sn=self.drive.serial_number,
                        ?error,
                        "failed to send cancel request to worker"
                    )
                });
        }

        tracing::info!(sn=self.drive.serial_number, "copy cancelled");

        resp.send(Ok(()))
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "CancelCopyDisc"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Check the status of the drive for stale data.
    ///
    /// If the drive has not received a status update from the drive monitor task or a running copy
    /// operation in a while, the status will be set to disconnected.
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn check_status(&mut self, resp: Response<()>) -> Result<()> {
        let reply = match &self.drive.state {
            OpticalDriveState::Idle => {
                if self.last_update.elapsed() >= DRIVE_TIMEOUT {
                    tracing::info!(sn=self.drive.serial_number, "drive timeout reached");
                    self.drive.state = OpticalDriveState::Disconnected;
                }
                Ok(())
            },
            OpticalDriveState::Copying {
                stage,
                task,
                task_progress,
                subtask,
                subtask_progress,
                elapsed_time: _,
            } => {
                if self.last_update.elapsed() >= DRIVE_TIMEOUT {
                    tracing::info!(sn=self.drive.serial_number, "drive timeout reached");
                    todo!()
                } else {
                    self.drive.state = OpticalDriveState::Copying {
                        stage,
                        task: task.clone(),
                        task_progress: *task_progress,
                        subtask: subtask.clone(),
                        subtask_progress: *subtask_progress,
                        elapsed_time: self.compute_elapsed_time(),
                    };
                }
                Ok(())
            },
            _ => Ok(()),
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "CheckDriveStatus"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Set the state of the drive to [`OpticalDriveState::Success`].
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn copy_completed(&mut self, resp: Response<()>) -> Result<()> {
        self.copy_started = None;
        self.copy_ct = None;

        if self.makemkv_copy_resp.is_some() {
            tracing::warn!(sn=self.drive.serial_number, "copy resp not none");
        }

        if self.makemkv_info_resp.is_some() {
            tracing::warn!(sn=self.drive.serial_number, "info resp not none");
        }

        let reply = if self.drive.state.is_copying() {
            self.drive.state = OpticalDriveState::Success;
            Ok(())
        } else {
            Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "CopyCompleted"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Set the state of the drive to [`OpticalDriveState::Failed`].
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn copy_failed(&mut self, error: String, resp: Response<()>) -> Result<()> {
        self.copy_started = None;
        self.copy_ct = None;

        if self.makemkv_copy_resp.is_some() {
            tracing::warn!(sn=self.drive.serial_number, "copy resp not none");
        }

        if self.makemkv_info_resp.is_some() {
            tracing::warn!(sn=self.drive.serial_number, "info resp not none");
        }

        let reply = if self.drive.state.is_copying() {
            self.drive.state = OpticalDriveState::Failed { error };
            Ok(())
        } else {
            Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "CopyCompleted"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Calculates the elapsed time of a running copy operation.
    fn compute_elapsed_time(&self) -> Duration {
        match self.copy_started {
            Some(started) => Instant::now() - started,
            None => Duration::default(),
        }
    }

    /// Gets the current drive status.
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response. See [`drive::get`] for
    /// more information on the response, including potential errors that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn get_status(&self, resp: Response<OpticalDrive>) -> Result<()> {
        resp.send(Ok(self.drive.clone()))
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "GetStatus"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Update copy operation progress based on the process of the current MakeMKV command.
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
    fn makemkv_progress(
        &mut self,
        op: String,
        op_prog: u8,
        subop: String,
        subop_prog: u8,
        resp: Response<()>
    ) -> Result<()> {
        let OpticalDriveState::Copying { stage, .. } = &self.drive.state else {
            let error = Error::InvalidDriveState { state: self.drive.state.name().to_owned() };
            return resp.send(Err(error))
                .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvProgress"))
                .map_err(|_| Error::ResponseSend);
        };

        self.drive.state = OpticalDriveState::Copying {
            stage,
            task: op,
            task_progress: (op_prog as f32) / 100.0,
            subtask: subop,
            subtask_progress: (subop_prog as f32) / 100.0,
            elapsed_time: self.compute_elapsed_time(),
        };

        resp.send(Ok(()))
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvProgress"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Loads the saved copy parameters for the drive.
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::read_form_data`] for more information on the response, including potential errors
    /// that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn read_form_data(&self, resp: Response<FormData>) -> Result<()> {
        let data = data::get_form_data(&self.drive.serial_number);
        resp.send(data)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "ReadFormData"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Resets the drive's state back to `Idle`.
    ///
    /// # Args
    ///
    /// `resp`:  The transmission end of the channel to send the response. See [`drive::reset`] for
    /// more information on the response, including potential errors that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn reset(&mut self, resp: Response<()>) -> Result<()> {
        let reply = match self.drive.state {
            OpticalDriveState::Success | OpticalDriveState::Failed { .. } => {
                self.drive.state = OpticalDriveState::Idle;
                tracing::info!(sn=self.drive.serial_number, "drive reset");
                Ok(())
            },
            _ => {
                Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
            },
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "Reset"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Runs the MakeMKV copy command to copy the titles on the disc to the file system.
    ///
    /// # Args
    ///
    /// `output_dir`:  The directory location where the video files should be written to.
    ///
    /// `log_file`:  The file location where the output of the command should be logged to.
    ///
    /// `ct`:  Cancellation token used to cancel the copy operation. It is assumed that the token
    /// is not already cancelled.
    ///
    /// `resp`:  Channel used to send the result of the command once its complete.
    ///
    /// # Errors
    ///
    /// [`Error::MakeMkv`] if an error occures while running the MakeMKV command.
    async fn run_makemkv_copy(
        &mut self,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        ct: CancellationToken,
        resp: Response<CopyCommandOutput>,
    ) -> Result<()> {
        let OpticalDriveState::Copying { .. } = &self.drive.state else {
            let error = Error::InvalidDriveState { state: self.drive.state.name().to_owned() };
            return resp.send(Err(error))
                .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvProgress"))
                .map_err(|_| Error::ResponseSend);
        };

        self.drive.state = OpticalDriveState::Copying {
            stage: "Copying Disc",
            task: String::default(),
            task_progress: 0.0,
            subtask: String::default(),
            subtask_progress: 0.0,
            elapsed_time: self.compute_elapsed_time(),
        };

        if self.makemkv_info_resp.is_some() || self.makemkv_copy_resp.is_some() {
            return resp.send(Err(Error::AlreadyRunning))
                .inspect_err(|_| send_error_trace(&self.drive.serial_number, "RunMakeMkvCopy"))
                .map_err(|_| Error::ResponseSend);
        };

        self.makemkv_copy_resp = Some(resp);

        match &self.worker {
            Some(worker) => {
                net::send_run_makemkv_copy(
                    &self.bus,
                    worker,
                    &self.drive.serial_number,
                    output_dir,
                    log_file,
                ).await
            },
            None => {
                drive::makemkv::run_makemkv_copy(
                    &self.bus,
                    &self.drive.serial_number,
                    &self.drive.path,
                    output_dir,
                    log_file,
                    ct,
                )
            }
        }
    }

    /// Notify the actor the MakeMKV copy command was completed successfully.
    ///
    /// # Args
    ///
    /// `output`:  The output of the copy command.
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::makemkv_copy_complete`] for more information on the response, including potential
    /// errors that could result.
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn makemkv_copy_complete(
        &mut self,
        output: CopyCommandOutput,
        resp: Response<()>,
    ) -> Result<()> {
        let reply = if let Some(copy_resp) = self.makemkv_copy_resp.take() {
            let _ = copy_resp.send(Ok(output))
                .inspect_err(|_| {
                    tracing::error!(sn=self.drive.serial_number, "failed to send copy result");
                });
            Ok(())
        } else {
            tracing::error!(sn=self.drive.serial_number, "copy command not running");
            Err(Error::NotRunning)
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakemkvCopyComplete"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Runs the MakeMKV info command to gather information about the disc's titles.
    ///
    /// # Args
    ///
    /// `log_file`:  The file location where the output of the command should be logged to.
    ///
    /// `ct`:  Cancellation token used to cancel the copy operation. It is assumed that the token
    /// is not already cancelled.
    ///
    /// `response`:  Channel used to send the result of the command once its complete. This will
    /// include the extracted disc information.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidMediaLocation`] if the provided log file location isn't valid
    ///
    /// [`Error::UnsupportedRequest`] if this request is made before the locality is known.
    async fn run_makemkv_info(
        &mut self,
        log_file: MediaLocation,
        ct: CancellationToken,
        resp: Response<InfoCommandOutput>,
    ) -> Result<()> {
        let OpticalDriveState::Copying { .. } = &self.drive.state else {
            let error = Error::InvalidDriveState { state: self.drive.state.name().to_owned() };
            return resp.send(Err(error))
                .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakeMkvProgress"))
                .map_err(|_| Error::ResponseSend);
        };

        self.drive.state = OpticalDriveState::Copying {
            stage: "Gathering Disc Info",
            task: String::default(),
            task_progress: 0.0,
            subtask: String::default(),
            subtask_progress: 0.0,
            elapsed_time: self.compute_elapsed_time(),
        };

        if self.makemkv_info_resp.is_some() || self.makemkv_copy_resp.is_some() {
            return resp.send(Err(Error::AlreadyRunning))
                .inspect_err(|_| send_error_trace(&self.drive.serial_number, "RunMakeMkvCopy"))
                .map_err(|_| Error::ResponseSend);
        };

        self.makemkv_info_resp = Some(resp);

        match &self.worker {
            Some(worker) => {
                net::send_run_makemkv_info(
                    &self.bus,
                    worker,
                    &self.drive.serial_number,
                    log_file
                ).await
            },
            None => {
                drive::makemkv::run_makemkv_info(
                    &self.bus,
                    &self.drive.serial_number,
                    &self.drive.path,
                    log_file,
                    ct,
                )
            },
        }
    }

    /// Notify the actor the MakeMKV info command was completed successfully.
    ///
    /// # Args
    ///
    /// `output`:  The output of the info command.
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::makemkv_info_complete`] for more information on the response, including potential
    /// errors that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn makemkv_info_complete(
        &mut self,
        output: InfoCommandOutput,
        resp: Response<()>,
    ) -> Result<()> {
        let reply = if let Some(info_resp) = self.makemkv_info_resp.take() {
            let _ = info_resp.send(Ok(output))
                .inspect_err(|_| {
                    tracing::error!(sn=self.drive.serial_number, "failed to send copy result");
                });
            Ok(())
        } else {
            tracing::error!(sn=self.drive.serial_number, "info command not running");
            Err(Error::NotRunning)
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakemkvCopyComplete"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Notify the actor that a MakeMKV command has failed.
    ///
    /// # Args
    ///
    /// `error`:  The command's error.
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::makemkv_info_complete`] for more information on the response, including potential
    /// errors that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn makemkv_failed(&mut self, error: String, resp: Response<()>) -> Result<()> {
        let mut running = false;

        if let Some(info_resp) = self.makemkv_info_resp.take() {
            let error = Error::MakeMkvCommandFailed { error: error.clone() };
            let _ = info_resp.send(Err(error))
                .inspect_err(|_| {
                    tracing::error!(sn=self.drive.serial_number, "failed to send info failure");
                });
            running = true;
        }

        if let Some(copy_resp) = self.makemkv_info_resp.take() {
            let error = Error::MakeMkvCommandFailed { error: error.clone() };
            let _ = copy_resp.send(Err(error))
                .inspect_err(|_| {
                    tracing::error!(sn=self.drive.serial_number, "failed to send copy failure");
                });
            running = true;
        }

        let reply = if running || self.copy_ct.is_none() {
            Ok(())
        } else {
            tracing::error!(sn=self.drive.serial_number, "command not running");
            Err(Error::NotRunning)
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "MakemkvFailed"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Saves the copy parameters for the drive.
    ///
    /// # Args
    ///
    /// `updated_date`:  The updated form data. Only the fields that are not `None` within this
    /// data should be updated.
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`drive::save_form_data`] for more information on the response, including potential errors
    /// that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn save_form_data(&self, updated_data: FormDataUpdate, resp: Response<()>) -> Result<()> {
        let mut data = match data::get_data(&self.drive.serial_number) {
            Ok(data) => data,
            Err(error) => {
                return resp.send(Err(error))
                    .inspect_err(|_| send_error_trace(&self.drive.serial_number, "SaveFormData"))
                    .map_err(|_| Error::ResponseSend);
            },
        };

        let mut should_save = false;

        if let Some(media_type) = updated_data.media_type {
            data.form.media_type = media_type;
            should_save = true;
        };

        if let Some(title) = updated_data.title {
            data.form.title = title;
            should_save = true;
        };

        if let Some(year) = updated_data.year {
            data.form.year = year;
            should_save = true;
        };

        if let Some(disc_number) = updated_data.disc_number {
            data.form.disc_number = disc_number;
            should_save = true;
        };

        if let Some(season_number) = updated_data.season_number {
            data.form.season_number = season_number;
            should_save = true;
        };

        if let Some(storage_location) = updated_data.storage_location {
            data.form.storage_location = storage_location;
            should_save = true;
        };

        if let Some(memo) = updated_data.memo {
            data.form.memo = memo;
            should_save = true;
        };

        let reply = if should_save {
            data::save_data(&self.drive.serial_number, &data)
        } else {
            Ok(())
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "SaveFormData"))
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

    /// Update drive information based on information from the OS.
    ///
    /// This will always update the path, hostname, and disc state. It will only update the drive
    /// state when the current state is `Disconnected` or `Idle` and the updated state is `Idle`
    /// or `Disconnected` respectively.
    ///
    /// The name and serial number will never be updated.
    ///
    /// # Args
    ///
    /// `drive`:  The optical drive information reported by the OS.
    ///
    /// `worker`:  The IP address of the worker node that sent the update. Will be `None` if the
    /// update was made from the same application instance.
    ///
    /// `resp`:  The transmission end of the channel to send the response. See
    /// [`crate::drive::update_from_os`] for more information on the response, including potential
    /// errors that could result.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn update_from_os(
        &mut self,
        drive: OsOpticalDrive,
        worker: Option<String>,
        resp: Response<()>
    ) -> Result<()> {
        self.last_update = Instant::now();

        // Only update fields associated with info provided by the OS that can change. Serial
        // number should be constant for a drive.

        if self.worker != worker {
            self.worker = worker;
        }

        if self.drive.path != drive.path {
            self.drive.path = drive.path;
        }

        if self.drive.hostname != drive.hostname {
            self.drive.hostname = drive.hostname;
        }

        if self.drive.disc != drive.disc {
            self.drive.disc = drive.disc;
        }

        // Only need change states if currently disconnected since getting a status update means
        // the drive is no longer considered disconnected.
        if self.drive.state == OpticalDriveState::Disconnected {
            self.drive.state = OpticalDriveState::Idle;
        }

        resp.send(Ok(()))
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "UpdateFromOs"))
            .map_err(|_| Error::ResponseSend)
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        let request = msg.drive_request(&self.drive.serial_number)?;

        match request {
            DriveRequest::BeginCopyDisc { params, response } => {
                self.begin_copy_disc(params, response)
            },
            DriveRequest::CancelCopyDisc { response } => {
                self.cancel_copy_disc(response).await
            },
            DriveRequest::CopyCompleted { response } => {
                self.copy_completed(response)
            },
            DriveRequest::CopyFailed { error, response } => {
                self.copy_failed(error, response)
            },
            DriveRequest::CheckDriveStatus { response } => {
                self.check_status(response)
            },
            DriveRequest::GetStatus { response } => {
                self.get_status(response)
            },
            DriveRequest::MakeMkvCopyComplete { output, response } => {
                self.makemkv_copy_complete(output, response)
            },
            DriveRequest::MakeMkvFailed { error, response } => {
                self.makemkv_failed(error, response)
            },
            DriveRequest::MakeMkvInfoComplete { output, response } => {
                self.makemkv_info_complete(output, response)
            },
            DriveRequest::MakeMkvProgress { op, op_prog, subop, subop_prog, response } => {
                self.makemkv_progress(op, op_prog, subop, subop_prog, response)
            },
            DriveRequest::ReadFormData { response } => {
                self.read_form_data(response)
            },
            DriveRequest::Reset { response } => {
                self.reset(response)
            },
            DriveRequest::RunMakeMkvCopy {
                output_dir,
                log_file,
                cancellation_token,
                response,
            } => {
                self.run_makemkv_copy(output_dir, log_file, cancellation_token, response).await
            },
            DriveRequest::RunMakeMkvInfo {
                log_file,
                cancellation_token,
                response
            } => {
                self.run_makemkv_info(log_file, cancellation_token, response).await
            },
            DriveRequest::SaveFormData { data, response } => {
                self.save_form_data(data, response)
            },
            DriveRequest::UpdateFromOs { drive, response, worker } => {
                self.update_from_os(drive, worker, response)
            },
            DriveRequest::WorkerMakeMkvCancel { response } => {
                self.unsupported_request("WorkerMakeMkvCancel", response)
            },
            DriveRequest::WorkerRunMakeMkvCopy { output_dir: _, log_file: _, response } => {
                self.unsupported_request("WorkerRunMakeMkvCopy", response)
            },
            DriveRequest::WorkerRunMakeMkvInfo { log_file: _, response } => {
                self.unsupported_request("WorkerRunMakeMkvInfo", response)
            },
        }
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
