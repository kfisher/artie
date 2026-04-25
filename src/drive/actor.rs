// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use tokio_util::sync::CancellationToken;

use makemkv::{CommandOutput, CopyCommandOutput, InfoCommandOutput};

use crate::{Error, Result};
use crate::actor::{self, Response};
use crate::bus;
use crate::drive::{
    FormData,
    FormDataUpdate,
    Handle,
    Message,
    OpticalDrive,
    OpticalDriveState,
    OsOpticalDrive,
};
use crate::drive::copy;
use crate::drive::data::Data;
use crate::models::{CopyParamaters, MediaLocation};
use crate::path;
use crate::task;

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

    /// Get the current status of an optical drive.
    GetStatus {
        response: Response<OpticalDrive>,
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

    /// Request to run the MakeMKV info command to gather information about the titles on the disc.
    RunMakeMkvInfo {
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: String,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: Response<InfoCommandOutput>,
    },

    /// Request to run the MakeMKV copy command to copy titles from the disc to the file system.
    RunMakeMkvCopy {
        command_output: mpsc::UnboundedSender<CommandOutput>,
        device_path: String,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        cancellation_token: CancellationToken,
        response: Response<CopyCommandOutput>,
    },

    /// Update the copy parameters stored in the drive's persistent data.
    SaveFormData {
        data: FormDataUpdate,
        response: Response<()>,
    },

    /// Updates the current state of the drive.
    ///
    /// This will update the drive status information based on an in-progress copy operation or an
    /// operation that completed, failed, or was cancelled.
    UpdateFromCopy {
        state: OpticalDriveState,
        response: Response<()>,
    },

    /// Updates the current state of the drive.
    ///
    /// This will update the drive status information based on what is reported by the OS. It is 
    /// mainly meant for use within the drive module only which is why there isn't a corresponding
    /// helper function.
    ///
    /// If `info` is `None`, then the information was unavailable without any errors being
    /// reported. This most likely means the drive is disconnected and will be treated as such.
    UpdateFromOs {
        info: Option<OsOpticalDrive>,
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
/// `drive`:  The drive the actor is being created for.
pub fn init(bus: bus::Handle, drive: OsOpticalDrive) -> Handle {
    let msg_processor = MessageProcessor::new(bus.clone(), drive);
    let name = format!("drive {}", &msg_processor.drive.serial_number);
    actor::create_and_run(&name, msg_processor)
}

/// Processes messages sent to the drive actor.
struct MessageProcessor {
    /// Handle used to send messages to other actors via the message bus.
    bus: bus::Handle,

    /// The drive's current state.
    drive: OpticalDrive,

    /// Cancellation token used to cancel a copy operation.
    ///
    /// This will only be `Some` during a copy operation.
    copy_ct: Option<CancellationToken>,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    ///
    /// # Args
    ///
    /// `bus`:  Handle used to send messages to other actors via the message bus.
    ///
    /// `drive`:  The optical drive associated with the actor instance that this message processor
    /// will be processing messages for.
    fn new(bus: bus::Handle, drive: OsOpticalDrive) -> Self {
        Self { 
            bus,
            drive: OpticalDrive::from_os(drive),
            copy_ct: None,
        }
    }

    /// Begin copying a disc.
    fn begin_copy_disc(&mut self, params: CopyParamaters, resp: Response<()>) -> Result<()> {
        let reply = if self.drive.state == OpticalDriveState::Idle {
            self.drive.state = OpticalDriveState::Copying {
                stage: String::from(""),
                task: String::from(""),
                task_progress: 0.0,
                subtask: String::from(""),
                subtask_progress: 0.0,
                elapsed_time: Duration::ZERO,
            };

            self.copy_ct = Some(CancellationToken::new());

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
    fn cancel_copy_disc(&mut self, resp: Response<()>) -> Result<()> {
        let reply = match self.drive.state {
            OpticalDriveState::Copying { .. } => {
                if let Some(copy_ct) = self.copy_ct.as_ref() {
                    copy_ct.cancel();
                    self.copy_ct = None;
                    tracing::info!(sn=self.drive.serial_number, "copy cancelled");
                    Ok(())
                } else {
                    Err(Error::CancelTokenNone)
                }
            },
            _ => {
                Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
            },
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "CancelCopyDisc"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Load the drive's persistent data.
    ///
    /// # Errors
    ///
    /// See [`Data::load`] for errors that can occur when attempting to read the data file.
    fn get_data(&self) -> Result<Data> {
        let path = self.get_data_path();
        Data::load(&path)
            .or_else(|error| {
                // File not being found is not an error.
                if let Error::FileNotFound { path } = error {
                    tracing::debug!(
                        serial_number=self.drive.serial_number,
                        ?path,
                        "drive data file not found"
                    );
                    Ok(Data::default())
                } else {
                    Err(error)
                }
            })
    }

    /// Gets the path to where the drive's persistent data is stored.
    fn get_data_path(&self) -> PathBuf {
        let name = format!("drive.{}.json", self.drive.serial_number);
        path::data_path(&name)
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
        let data = self.get_data()
            .map(|data| data.form);
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

    /// Runs the MakeMKV info command to gather information about the disc's titles.
    ///
    /// # Args
    ///
    /// `cmd_output`:  Channel used by the MakeMKV command to relay output from the command as well
    /// as progress information.
    ///
    /// `device`:  Device path (or name) of the optical drive to perform the operation on
    /// (e.g. "/dev/sr0").
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
    /// [`Error::MakeMkv`] if an error occures while running the MakeMKV command.
    fn run_makemkv_info(
        &self,
        cmd_output: mpsc::UnboundedSender<CommandOutput>,
        device: String,
        log_file: MediaLocation,
        ct: CancellationToken,
        response: oneshot::Sender<Result<InfoCommandOutput>>,
    ) -> Result<()> {
        let log_path = path::location_path(&log_file)
            .ok_or(Error::InvalidMediaLocation { location: log_file })?;

        task::spawn(async move {
            let output = makemkv::get_disc_info(&device, &cmd_output, &log_path, &ct)
                .await
                .map_err(|e| e.into());
            if let Err(error) = response.send(output) {
                tracing::error!(?error, "failed to send info command response");
            }
        });

        Ok(())
    }

    /// Runs the MakeMKV copy command to copy the titles on the disc to the file system.
    ///
    /// # Args
    ///
    /// `cmd_output`:  Channel used by the MakeMKV command to relay output from the command as well
    /// as progress information.
    ///
    /// `device`:  Device path (or name) of the optical drive to perform the copy operation on
    /// (e.g. "/dev/sr0").
    ///
    /// `output_dir`:  The directory location where the video files should be written to.
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
    /// [`Error::InvalidMediaLocation`] if one of the provided media locations is invalid.
    ///
    /// [`Error::MakeMkv`] if an error occures while running the MakeMKV command.
    fn run_makemkv_copy(
        &self,
        cmd_output: mpsc::UnboundedSender<CommandOutput>,
        device: String,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        ct: CancellationToken,
        response: oneshot::Sender<Result<CopyCommandOutput>>,
    ) -> Result<()> {
        let output_path = path::location_path(&output_dir)
            .ok_or(Error::InvalidMediaLocation { location: output_dir })?;
        let log_path = path::location_path(&log_file)
            .ok_or(Error::InvalidMediaLocation { location: log_file })?;

        task::spawn(async move {
            let output = makemkv::copy_disc(&device, &output_path, &cmd_output, &log_path, &ct)
                .await
                .map_err(|e| e.into());
            if let Err(error) = response.send(output) {
                tracing::error!(?error, "failed to send copy command response");
            }
        });

        Ok(())
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
        let mut data = self.get_data()?;

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
            let path = self.get_data_path();
            data.save(&path)
        } else {
            Ok(())
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "SaveFormData"))
            .map_err(|_| Error::ResponseSend)
    }

    /// Update the current state of the drive.
    ///
    /// This will update the drive status information based on an in-progress copy operation or an
    /// operation that completed, failed, or was cancelled.
    ///
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn update_from_copy(&mut self, state: OpticalDriveState, resp: Response<()>) -> Result<()> {
        // If our current state is Disconnected, then we shouldn't be running a copy operation. If
        // the drive is disconnected while running a copy operation, its expected to go to the
        // failed state instead.
        //
        // If the current state is Success or Failed, then the state should be reset using the
        // reset request before starting another copy operation.
        let reply = match &self.drive.state {
            OpticalDriveState::Disconnected => {
                Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
            },
            OpticalDriveState::Success => {
                Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
            },
            OpticalDriveState::Failed { error: _ } => {
                Err(Error::InvalidDriveState { state: self.drive.state.name().to_owned() })
            },
            _ => {
                self.drive.state = state;
                Ok(())
            },
        };

        resp.send(reply)
            .inspect_err(|_| send_error_trace(&self.drive.serial_number, "UpdateFromCopy"))
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
    /// # Errors
    ///
    /// [`Error::ResponseSend`] if the response cannot be sent.
    fn update_from_os(&mut self, info: Option<OsOpticalDrive>, resp: Response<()>) -> Result<()> {
        let result = if let Some(info) = info {
            // Only update fields associated with info provided by the OS that can change. Serial
            // number should be constant for a drive.
            self.drive.path = info.path;
            self.drive.hostname = info.hostname;
            self.drive.disc = info.disc;
            if self.drive.state == OpticalDriveState::Disconnected {
                self.drive.state = OpticalDriveState::Idle;
            }
            Ok(())
        } else {
            if self.drive.state == OpticalDriveState::Idle {
                self.drive.state = OpticalDriveState::Disconnected;
            }
            Ok(())
        };

        resp.send(result)
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
                self.cancel_copy_disc(response)
            },
            DriveRequest::GetStatus { response } => {
                self.get_status(response)
            },
            DriveRequest::ReadFormData { response } => {
                self.read_form_data(response)
            },
            DriveRequest::Reset { response } => {
                self.reset(response)
            },
            DriveRequest::RunMakeMkvCopy {
                command_output,
                device_path,
                output_dir,
                log_file,
                cancellation_token,
                response,
            } => {
                self.run_makemkv_copy(
                    command_output,
                    device_path,
                    output_dir,
                    log_file,
                    cancellation_token,
                    response
                )
            },
            DriveRequest::RunMakeMkvInfo {
                command_output,
                device_path,
                log_file,
                cancellation_token,
                response
            } => {
                self.run_makemkv_info(
                    command_output,
                    device_path,
                    log_file,
                    cancellation_token,
                    response,
                )
            },
            DriveRequest::SaveFormData { data, response } => {
                self.save_form_data(data, response)
            },
            DriveRequest::UpdateFromCopy { state, response } => {
                self.update_from_copy(state, response )
            },
            DriveRequest::UpdateFromOs { info, response } => {
                self.update_from_os(info, response)
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
