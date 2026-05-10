// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles running MakeMKV operations for a drive.

use tokio::sync::mpsc;

use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;

use makemkv::CommandOutput;

use crate::{Error, Result};
use crate::bus;
use crate::drive;
use crate::path;
use crate::models::MediaLocation;
use crate::task;

/// Runs the MakeMKV copy command to copy the titles on the disc to the file system.
///
/// # Args
///
/// `bus`:  Handle used to send messages to actors via the message bus.
///
/// `serial_number`:  The serial number of the optical drive to run the MakeMKV command on.
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
/// # Errors
///
/// [`Error::InvalidMediaLocation`] if one of the provided media locations are invalid.
pub fn run_makemkv_copy(
    bus: &bus::Handle,
    serial_number: &str,
    device: &str,
    output_dir: MediaLocation,
    log_file: MediaLocation,
    cancellation_token: CancellationToken,
) -> Result<()> {
    let output_path = path::location_path(&output_dir)
        .ok_or(Error::InvalidMediaLocation { location: output_dir })?;
    let log_path = path::location_path(&log_file)
        .ok_or(Error::InvalidMediaLocation { location: log_file })?;

    let (tx, rx) = mpsc::unbounded_channel::<CommandOutput>();
    let ct = cancellation_token.clone();
    task::spawn(
        process_command_output(bus.clone(), serial_number.to_owned(), rx)
            .with_cancellation_token_owned(ct)
    );

    let bus = bus.clone();
    let serial_number = serial_number.to_owned();
    let device = device.to_owned();
    task::spawn(async move {
        match makemkv::copy_disc(&device, &output_path, &tx, &log_path, &cancellation_token).await {
            Ok(output) => {
                drive::makemkv_copy_complete(&bus, &serial_number, output).await
                    .inspect_err(|_| {
                        tracing::error!(sn=serial_number, "failed to send copy complete");
                    })
            },
            Err(error) => {
                drive::makemkv_failed(&bus, &serial_number, format!("{:?}", error)).await
                    .inspect_err(|_| {
                        tracing::error!(sn=serial_number, "failed to send copy failed");
                    })
            },
        }
    });

    Ok(())
}

/// Runs the MakeMKV info command to gather information about the disc's titles.
///
/// # Args
///
/// `bus`:  Handle used to send messages to actors via the message bus.
///
/// `serial_number`:  The serial number of the optical drive to run the MakeMKV command on.
///
/// `device`:  Device path (or name) of the optical drive to perform the operation on
/// (e.g. "/dev/sr0").
///
/// `log_file`:  The file location where the output of the command should be logged to.
///
/// `ct`:  Cancellation token used to cancel the copy operation. It is assumed that the token is
/// not already cancelled.
///
/// # Errors
///
/// [`Error::InvalidMediaLocation`] if the provided log file location isn't valid
pub fn run_makemkv_info(
    bus: &bus::Handle,
    serial_number: &str,
    device: &str,
    log_file: MediaLocation,
    cancellation_token: CancellationToken,
) -> Result<()> {
    let log_path = path::location_path(&log_file)
        .ok_or(Error::InvalidMediaLocation { location: log_file })?;

    let (tx, rx) = mpsc::unbounded_channel::<CommandOutput>();
    let ct = cancellation_token.clone();
    task::spawn(
        process_command_output(bus.clone(), serial_number.to_owned(), rx)
            .with_cancellation_token_owned(ct)
    );

    let bus = bus.clone();
    let serial_number = serial_number.to_owned();
    let device = device.to_owned();
    task::spawn(async move { let tx = tx.clone();
        match makemkv::get_disc_info(&device, &tx, &log_path, &cancellation_token).await {
            Ok(output) => {
                drive::makemkv_info_complete(&bus, &serial_number, output).await
                    .inspect_err(|_| {
                        tracing::error!(sn=serial_number, "failed to send info complete");
                    })
            },
            Err(error) => {
                drive::makemkv_failed(&bus, &serial_number, format!("{:?}", error)).await
                    .inspect_err(|_| {
                        tracing::error!(sn=serial_number, "failed to send info failed");
                    })
            },
        }
    });

    Ok(())
}

/// Processes the output of a running MakeMKV command.
///
/// # Args
///
/// `bus`:  Handle used to send messages to actors via the message bus.
///
/// `serial_number`:  The serial number of the optical drive the MakeMKV command is running on.
///
/// `rx`:  Receiving end of the channel output from the running command is written to.
async fn process_command_output(
    bus: bus::Handle,
    serial_number: String,
    mut rx: mpsc::UnboundedReceiver<CommandOutput>
) {
    while let Some(data) = rx.recv().await {
        match data {
            CommandOutput::Message(_message) => {
                // TODO: Handle MakeMKV general messages.
            },
            CommandOutput::Progress(progress) => {
                let result = drive::makemkv_progress(
                    &bus,
                    &serial_number,
                    progress.op,
                    progress.op_prog,
                    progress.subop,
                    progress.subop_prog,
                ).await;
                if let Err(error) = result {
                    tracing::error!(sn=serial_number, ?error, "failed to send progress update");
                }
            },
            CommandOutput::Error(_error) => {
                // TODO: Handle MakeMKV error output.
            },
        }
    }
}


#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
