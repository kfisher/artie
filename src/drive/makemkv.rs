// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles running MakeMKV operations for a drive.

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use tokio_util::sync::CancellationToken;

use makemkv::{CommandOutput, CopyCommandOutput, InfoCommandOutput};

use crate::{Error, Result};
use crate::path;
use crate::models::MediaLocation;
use crate::task;

/// Runs the MakeMKV copy command to copy the titles on the disc to the file system.
///
/// # Args
///
/// `cmd_output`:  Channel used by the MakeMKV command to relay output from the command as well as
/// progress information.
///
/// `device`:  Device path (or name) of the optical drive to perform the copy operation on
/// (e.g. "/dev/sr0").
///
/// `output_dir`:  The directory location where the video files should be written to.
///
/// `log_file`:  The file location where the output of the command should be logged to.
///
/// `ct`:  Cancellation token used to cancel the copy operation. It is assumed that the token is
/// not already cancelled.
///
/// `response`:  Channel used to send the result of the command once its complete. This will
/// include the extracted disc information.
///
/// # Errors
///
/// [`Error::InvalidMediaLocation`] if one of the provided media locations is invalid.
///
/// [`Error::MakeMkv`] if an error occures while running the MakeMKV command.
pub(super) fn run_makemkv_copy(
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

/// Runs the MakeMKV info command to gather information about the disc's titles.
///
/// # Args
///
/// `cmd_output`:  Channel used by the MakeMKV command to relay output from the command as well as
/// progress information.
///
/// `device`:  Device path (or name) of the optical drive to perform the operation on
/// (e.g. "/dev/sr0").
///
/// `log_file`:  The file location where the output of the command should be logged to.
///
/// `ct`:  Cancellation token used to cancel the copy operation. It is assumed that the token is
/// not already cancelled.
///
/// `response`:  Channel used to send the result of the command once its complete. This will
/// include the extracted disc information.
///
/// # Errors
///
/// [`Error::InvalidMediaLocation`] if the provided log file location isn't valid
///
/// [`Error::MakeMkv`] if an error occures while running the MakeMKV command.
pub(super) fn run_makemkv_info(
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
