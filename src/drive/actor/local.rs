// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;

use makemkv::{CommandOutput, CopyCommandOutput, InfoCommandOutput};

use crate::Result;
use crate::db::Database;
use crate::drive;
use crate::drive::{DiscState, OpticalDrive, OpticalDriveState, OpticalDriveStatus};
use crate::drive::actor::DriveActorMessage;
use crate::drive::actor::handle::DriveActorHandle;
use crate::drive::copy;
use crate::drive::data::{Data, FormData, FormDataUpdate};
use crate::error::{Error, ChannelError};
use crate::fs::FileSystem;
use crate::models::{CopyParamaters, MediaLocation};
use crate::task;

/// Create a [`DriveActor`] instance, start it, and return its handle.
pub fn create_actor(drive: &OpticalDrive, fs: FileSystem, db: Database) -> DriveActorHandle {
    DriveActor::create(drive.clone(), fs, db)
}

/// Actor used to perform copy operations and monitor the state of an optical
// TODO: DOC
#[derive(Debug)]
struct DriveActor {
    /// The optical drive this actor is associated with.
    drive: OpticalDrive,

    /// The current state of the drive.
    state: OpticalDriveState,

    /// File system utilities.
    fs: FileSystem,

    /// Interface to the application's database.
    db: Database,

    /// Transmission end of the channel used to send requests to the actor.
    ///
    /// This isn't used directly by the actor. It is cloned when creating new handle instance from
    /// the actor.
    tx: mpsc::Sender<DriveActorMessage>,

    /// Receiving end of the channel used to send requests to the actor.
    rx: mpsc::Receiver<DriveActorMessage>,

    /// Cancellation token used to cancel a copy operation.
    copy_ct: Option<CancellationToken>,
}

impl DriveActor {
    /// Create a [`DriveActor`] instance.
    fn new(
        drive: OpticalDrive,
        fs: FileSystem,
        db: Database,
        tx: mpsc::Sender<DriveActorMessage>,
        rx: mpsc::Receiver<DriveActorMessage>,
    ) -> Self {
        Self { drive, state: OpticalDriveState::Idle, fs, db, tx, rx, copy_ct: None }
    }

    /// Create a [`DriveActor`] instance, start it, and return its handle.
    fn create(drive: OpticalDrive, fs: FileSystem, db: Database) -> DriveActorHandle {
        let (tx, rx) = mpsc::channel(10);

        let actor = DriveActor::new(drive, fs, db, tx.clone(), rx);
        task::spawn(run_actor(actor));

        DriveActorHandle::new(tx)
    }

    /// Cancels a copy operation currently in progress.
    fn cancel_copy_disc(&mut self) -> Result<()> {
        if let Some(copy_ct) = self.copy_ct.as_ref() {
            copy_ct.cancel();
            tracing::info!(sn=?self.drive.serial_number, "copy cancelled");
            self.copy_ct = None;
        } else {
            tracing::warn!(sn=?self.drive.serial_number, "failed to cancel copy operation");
        }

        Ok(())
    }

    /// Begins copying the disc in the optical drive.
    fn copy_disc(&mut self, copy_parameters: CopyParamaters) -> Result<()> {
        if self.state != OpticalDriveState::Idle {
            return Err(
                Error::InvalidOpticalDriveState {
                    state: Box::new(self.state.clone()),
                    expected: Box::new(OpticalDriveState::Idle),
                });
        }

        let state = OpticalDriveState::Copying {
            stage: String::from(""),
            task: String::from(""),
            task_progress: 0.0,
            subtask: String::from(""),
            subtask_progress: 0.0,
            elapsed_time: Duration::ZERO,
        };
        self.update_state(state)?;

        self.copy_ct = Some(CancellationToken::new());

        task::spawn(
            copy::copy_disc(
                self.drive.clone(),
                copy_parameters,
                self.fs.clone(),
                self.db.clone(),
                DriveActorHandle::new(self.tx.clone()),
                self.copy_ct.as_ref().unwrap().clone(),
            )
        );

        Ok(())
    }

    /// Process a message that was received from the actor's communication channel.
    fn handle_message(&mut self, msg: DriveActorMessage) -> Result<()> {
        match msg {
            DriveActorMessage::CancelCopyDisc => self.cancel_copy_disc(),
            DriveActorMessage::CopyDisc { parameters } => self.copy_disc(parameters),
            DriveActorMessage::GetFormData { response } => self.get_form_data(response),
            DriveActorMessage::GetStatus { response } => self.get_state(response),
            DriveActorMessage::Reset => self.reset(),
            DriveActorMessage::RunMakeMkvInfo {
                command_output,
                device_path,
                log_file,
                cancellation_token,
                response,
            } => {
                self.run_makemkv_info(
                    command_output,
                    device_path,
                    log_file,
                    cancellation_token,
                    response,
                )
            },
            DriveActorMessage::RunMakeMkvCopy {
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
                    response,
                )
            },
            DriveActorMessage::UpdateFormData { data } => self.update_form_data(data),
            DriveActorMessage::UpdateOpticalDriveState { state } => self.update_state(state),
        }
    }

    /// Loads the drive's persistent data and returns it.
    fn get_data(&self) -> Result<Data> {
        let path = self.get_data_path()?;
        Data::load(&path)
            .or_else(|e| {
                // File not being found is not an error.
                if let Error::FileNotFound { path } = e {
                    tracing::debug!(?path, "drive data file not found");
                    Ok(Data::default())
                } else {
                    Err(e)
                }
            })
    }

    /// Process the request for getting the form data from the drive's persistent data.
    fn get_form_data(&self, tx: oneshot::Sender<FormData>) -> Result<()> {
        let data = self.get_data()?;

        tx.send(data.form)
            .map_err(|_| Error::DriveActorChannel { error: Box::new(ChannelError::OneShotSend) })?;

        Ok(())
    }

    /// Process the request for getting the current state of the actor.
    fn get_state(&self, tx: oneshot::Sender<OpticalDriveStatus>) -> Result<()> {
        let status = match drive::get_optical_drive(&self.drive.serial_number)? {
            Some(drive) => OpticalDriveStatus::new(drive.disc, self.state.clone()),
            None => OpticalDriveStatus::new(DiscState::None, OpticalDriveState::Disconnected),
        };

        tx.send(status)
            .map_err(|_| Error::DriveActorChannel { error: Box::new(ChannelError::OneShotSend) })?;

        Ok(())
    }

    /// Reset the drive state back to `Idle` from `Success` or `Failed`.
    fn reset(&mut self) -> Result<()> {
        match &self.state {
            OpticalDriveState::Success | OpticalDriveState::Failed { .. } => {
                self.state = OpticalDriveState::Idle;
                tracing::info!(sn=?self.drive.serial_number, "drive reset");
            },
            _ => {
                tracing::warn!(
                    sn=?self.drive.serial_number,
                    state=?self.state,
                    "cannot reset drive"
                );
            }
        }

        Ok(())
    }

    /// Runs the MakeMKV info command to gather information about the disc's titles.
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
    /// [`Error::MakeMKV`] if an error occures while running the MakeMKV command.
    pub fn run_makemkv_info(
        &self,
        cmd_output: mpsc::UnboundedSender<CommandOutput>,
        device: String,
        log_file: MediaLocation,
        ct: CancellationToken,
        response: oneshot::Sender<Result<InfoCommandOutput>>,
    ) -> Result<()> {
        // TODO: See what happens if this is invalid.
        let log_path = self.fs.path(&log_file)
            .ok_or(Error::InvalidMediaLocation { location: log_file })?;

        task::spawn(async move {
            let output = makemkv::get_disc_info(&device, &cmd_output, &log_path, &ct)
                .await
                .map_err(|error| Error::MakeMKV { error });
            if let Err(error) = response.send(output) {
                tracing::error!(?error, "failed to send info command response");
            }
        });

        Ok(())
    }

    /// Runs the MakeMKV copy command to copy the titles on the disc to the file system.
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
    pub fn run_makemkv_copy(
        &self,
        cmd_output: mpsc::UnboundedSender<CommandOutput>,
        device: String,
        output_dir: MediaLocation,
        log_file: MediaLocation,
        ct: CancellationToken,
        response: oneshot::Sender<Result<CopyCommandOutput>>,
    ) -> Result<()> {
        let output_path = self.fs.path(&output_dir)
            .ok_or(Error::InvalidMediaLocation { location: output_dir })?;
        let log_path = self.fs.path(&log_file)
            .ok_or(Error::InvalidMediaLocation { location: log_file })?;

        task::spawn(async move {
            let output = makemkv::copy_disc(&device, &output_path, &cmd_output, &log_path, &ct)
                .await
                .map_err(|error| Error::MakeMKV { error });
            if let Err(error) = response.send(output) {
                tracing::error!(?error, "failed to send copy command response");
            }
        });

        Ok(())
    }

    /// Updates the form data in the drive's persistent data.
    fn update_form_data(&self, form_data: FormDataUpdate) -> Result<()> {
        let mut data = self.get_data()?;

        let mut should_save = false;

        if let Some(media_type) = form_data.media_type {
            data.form.media_type = media_type;
            should_save = true;
        };

        if let Some(title) = form_data.title {
            data.form.title = title;
            should_save = true;
        };

        if let Some(year) = form_data.year {
            data.form.year = year;
            should_save = true;
        };

        if let Some(disc_number) = form_data.disc_number {
            data.form.disc_number = disc_number;
            should_save = true;
        };

        if let Some(season_number) = form_data.season_number {
            data.form.season_number = season_number;
            should_save = true;
        };

        if let Some(storage_location) = form_data.storage_location {
            data.form.storage_location = storage_location;
            should_save = true;
        };

        if let Some(memo) = form_data.memo {
            data.form.memo = memo;
            should_save = true;
        };

        if should_save {
            let path = self.get_data_path()?;
            data.save(&path)?;
        }

        Ok(())
    }

    /// Update the state of the optical drive.
    fn update_state(&mut self, state: OpticalDriveState) -> Result<()> {
        self.state = state;
        Ok(())
    }

    /// Gets the path to where the drive's persistent data is stored.
    fn get_data_path(&self) -> Result<PathBuf> {
        let name = format!("drive.{}.json", self.drive.serial_number);
        Ok(self.fs.data_path(&name))
    }
}

/// Runs the processing loop for the provided actor.
async fn run_actor(mut actor: DriveActor) {
    tracing::info!(sn=actor.drive.serial_number, "message processing started");

    while let Some(msg) = actor.rx.recv().await {
        if let Err(error) = actor.handle_message(msg) {
            tracing::error!(sn=actor.drive.serial_number, ?error, "Failed to process message.");
        }
    }

    tracing::info!(sn=actor.drive.serial_number, "message processing ended");
}
