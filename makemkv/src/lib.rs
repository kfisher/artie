// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for running the MakeMKV command.

mod commands;
mod data;
mod error;
mod messages;

#[cfg(test)]
mod test_utils;

use tokio::sync::mpsc::UnboundedSender;

use tokio_util::sync::CancellationToken;

pub use crate::data::{DiscInfo, StreamInfo, TitleInfo};
pub use crate::error::{Error, Result};
pub use crate::commands::CommandOutput;

use std::fs;
use std::path::Path;
use std::process::ExitStatus;

use crate::commands::Context;

/// Output data from copying a disc.
#[derive(Debug)]
pub struct CopyCommandOutput {
    /// Exit code for the operation.
    ///
    /// Note that MakeMKV will sometimes return a non-zero exit code even when the operation
    /// completed successfully.
    pub exit_status: ExitStatus,

    /// The raw output from the MakeMKV command.
    pub log: String,
}

/// Output data from gathering disc info.
#[derive(Debug)]
pub struct InfoCommandOutput {
    /// Exit code for the operation.
    ///
    /// Note that MakeMKV will sometimes return a non-zero exit code even when the operation
    /// completed successfully.
    pub exit_status: ExitStatus,

    /// The raw output from the MakeMKV command.
    pub log: String,

    /// The extract disc information.
    pub disc_info: DiscInfo,
}

/// Represents the progress of a MakeMKV command.
///
/// MakeMKV breaks each stage or operation down into suboperations when reporting status
/// information. The progress values range from 0 to 100 where 0 is 0% complete and 100 is 100%
/// complete. Its done this way to make UI and network related tasks easier. Since its really only
/// intended for display, it doesn't need the preciseness of a floating value.
#[derive(Clone, Debug)]
pub struct Progress {
    /// Title of the current operation.
    pub op: String,

    /// Progress of the current operation.
    pub op_prog: u8,

    /// Title of the current suboperation.
    pub subop: String,

    /// Progress of the current suboperation.
    pub subop_prog: u8,
}

impl Progress {
    /// Creates a new `Progress` instance with empty titles and 0% progress.
    fn new() -> Progress {
        Progress {
            op: String::new(),
            op_prog: 0,
            subop: String::new(),
            subop_prog: 0,
        }
    }
}

/// Extract information about a disc's content.
///
/// This will run the MakeMKV "info" command to extract information about the titles on the disc
/// inserted into the optical drive specified by the device path `device` and return that
/// information when it completes successfully.
///
/// `device` is the device path of the optical drive (e.g. "/dev/sr0").
///
/// `observer` provides the ability for the caller to receive messages while the command is
/// running with information messages and the current progress.
///
/// `log_path` path to where to save the raw output from HandBrake.
///
/// `ct` cancellation token used to cancel the disc info command before it completes.
pub async fn get_disc_info(
    device: &str,
    observer: &UnboundedSender<CommandOutput>,
    log_path: &Path,
    ct: &CancellationToken,
) -> Result<InfoCommandOutput>
{
    let mut ctx = Context::new(device, observer, ct.clone());
    ctx.log_output(log_path)?;

    let exit_status = commands::run_info_command(&mut ctx).await?;

    let log = fs::read_to_string(log_path)
        .map_err(|error| Error::FileOpenError { path: log_path.to_owned(), error })?;

    let disc_info = ctx.take_disc_info().ok_or(Error::MissingDiscInfo)?;

    let output = InfoCommandOutput {
        exit_status,
        log,
        disc_info,
    };

    Ok(output)
}

/// Copy the titles from a disc and save them as MKV files.
///
/// This will run the MakeMKV "mkv" command which will copy all titles that meet the minimum and
/// maximum length requirements to MKV files and save them to the provided output directory. The
/// length requirements are configured via MakeMKV and not adjustable by this program.
///
/// `dev` is the device path of the optical drive (e.g. "/dev/sr0").
///
/// `out_dir` is the directory where the created MKV files should be saved.
///
/// `observer` provides the ability for the caller to receive messages while the command is
/// running with information messages and the current progress.
///
/// `log_path` path to where to save the raw output from HandBrake.
///
/// `ct` cancellation token used to cancel the disc info command before it completes.
///
/// # Errors
///
/// In addition to the errors that can occur during the command's execution, this will error out if
/// there are any MKV files in the output directory.
pub async fn copy_disc(
    device: &str,
    out_dir: &Path,
    observer: &UnboundedSender<CommandOutput>,
    log_path: &Path,
    ct: &CancellationToken,
) -> Result<CopyCommandOutput>
{
    if contains_mkv_files(out_dir)? {
        return Err(Error::FoundExistingMkvFiles { path: out_dir.to_path_buf() });
    }

    let mut ctx = Context::new(device, observer, ct.clone());
    ctx.log_output(log_path)?;

    let exit_status = commands::run_mkv_command(&mut ctx, out_dir).await?;

    let log = fs::read_to_string(log_path)
        .map_err(|error| Error::FileOpenError { path: log_path.to_owned(), error })?;

    let output = CopyCommandOutput {
        exit_status,
        log,
    };

    Ok(output)
}

/// Checks for the existence of MKV files in a directory.
fn contains_mkv_files(dir: &Path) -> Result<bool> {
    for item in dir
        .read_dir()
        .map_err(|e| Error::ExistingMkvFilesCheckIoError { error: e })?
    {
        let item = item.map_err(|e| Error::ExistingMkvFilesCheckIoError { error: e })?;
        if item.path().extension().and_then(|s| s.to_str()) == Some("mkv") {
            return Ok(true);
        }
    }
    Ok(false)
}

