// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for running the MakeMKV command.

mod commands;
mod data;
mod error;
mod messages;

#[cfg(test)]
mod test_utils;

pub use crate::data::DiscInfo;
pub use crate::error::{Error, Result};

use std::path::Path;
use std::process::ExitStatus;

use crate::commands::Context;

/// Trait for receiving data from running commands.
///
/// When running commands, a struct that implements this trait can be provided so that the caller
/// can receive information from a running command such as its progress. Each function has a
/// default implementation that does nothing so the caller can choose to only implement the
/// functions they care about.
pub trait Observe {
    /// Called when a general information is received.
    fn message(&mut self, _msg: &str) {}

    /// Called when a progress update is received.
    fn progress(&mut self, _progress: &Progress) {}

    /// Called when output from the error console is received.
    fn error(&mut self, _err: &str) {}
}

/// Represents the progress of a MakeMKV command.
///
/// MakeMKV breaks each stage or operation down into suboperations when reporting status
/// information. The progress values range from 0 to 100 where 0 is 0% complete and 100 is 100%
/// complete. Its done this way to make UI and network related tasks easier. Since its really only
/// intended for display, it doesn't need the preciseness of a floating value.
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
/// <div class="warning">
///
/// Calling this function will block until the command completes. This can take a long time
/// depending on the options and the system its being run on.
///
/// </div>
///
/// This will run the MakeMKV "info" command to extract information about the titles on the disc
/// inserted into the optical drive specified by the device path `device` and return that
/// information when it completes successfully.
///
/// `dev` is the device path of the optical drive (e.g. "/dev/sr0").
///
/// `observer` provides the ability for the caller to receive callbacks while the handbrake is
/// running with information messages and the current progress.
///
/// `log_path` path to where to save the raw output from HandBrake.
pub fn get_disc_info<T>(
    device: &str,
    observer: &mut T,
    log_path: &Path
) -> Result<(ExitStatus, DiscInfo)>
where
    T: Observe,
{
    let mut ctx = Context::new(device, observer);
    ctx.log_output(log_path)?;

    let exit_status = commands::run_info_command(&mut ctx)?;

    let disc_info = ctx.take_disc_info().ok_or(Error::MissingDiscInfo)?;

    Ok((exit_status, disc_info))
}

/// Copy the titles from a disc and save them as MKV files.
///
/// <div class="warning">
///
/// Calling this function will block until the command completes. This can take a long time
/// depending on the options and the system its being run on.
///
/// </div>
///
/// This will run the MakeMKV "mkv" command which will copy all titles that meet the minimum and
/// maximum length requirements to MKV files and save them to the provided output directory. The
/// length requirements are configured via MakeMKV and not adjustable by this program.
///
/// `dev` is the device path of the optical drive (e.g. "/dev/sr0").
///
/// `out_dir` is the directory where the created MKV files should be saved.
///
/// `observer` provides the ability for the caller to receive callbacks while the handbrake is
/// running with information messages and the current progress.
///
/// `log_path` path to where to save the raw output from HandBrake.
///
/// # Errors
///
/// In addiiton to the errors that can occur during the command's execution, this will error out if
/// there are any MKV files in the output directory.
pub fn copy_disc<T>(
    device: &str,
    out_dir: &Path,
    observer: &mut T,
    log_path: &Path
) -> Result<ExitStatus>
where
    T: Observe,
{
    if contains_mkv_files(out_dir)? {
        return Err(Error::FoundExistingMkvFiles { path: out_dir.to_path_buf() });
    }

    let mut ctx = Context::new(device, observer);
    ctx.log_output(log_path)?;

    commands::run_mkv_command(&mut ctx, out_dir)
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
