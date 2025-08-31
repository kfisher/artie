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

use std::fs::OpenOptions;
use std::path::Path;

use crate::commands::{Context, OsRunner};

/// Filename where the "info" command log is written to.
pub const INFO_CMD_LOG_FILENAME: &str = "makemkv.info.log";

/// Filename where the "mkv" command log is written to.
pub const COPY_CMD_LOG_FILENAME: &str = "makemkv.mkv.log";

/// Filename where the disc information is written to.
pub const DISC_INFO_FILENAME: &str = "disc_info.json";

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
/// This will run the MakeMKV "info" command to extract information about the titles on the disc
/// inserted into the optical drive specified by the device path `device` and return that
/// information when it completes successfully.
///
/// In addition to returning the disc information, its content will be written to the provided
/// output directory in a file called "disc_info.json". Additionally, the output from MakeMKV will
/// be written to "makemkv.info.log".
///
/// This function will take a long time to complete.
///
/// It is expected that the provided output directory will already exist and that the directory
/// won't contain preexisting log or disc information files. If any of these expectations aren't
/// met, the function will return an error. It is up to the caller to create the directory and
/// delete existing files if required.
pub fn run_info_cmd<T>(device: &str, outdir: &Path, observer: &mut T) -> Result<DiscInfo>
where
    T: Observe,
{
    // NOTE: We don't check if the device path is valid/exists here since its likely to be OS
    //       dependent. If it happens to be invalid, the command should fail early. Will revisit if
    //       it becomes an issue in the future.
    //
    if !outdir.is_dir() {
        return Err(Error::OutputDirDoesNotExist(outdir.to_owned()));
    }

    let mut ctx = Context::new(device, outdir, observer);
    ctx.enable_cmd_log(INFO_CMD_LOG_FILENAME)?;

    commands::run_info_command::<OsRunner>(&mut ctx)?;

    let Some(disc_info) = ctx.take_disc_info() else {
        return Err(Error::MissingDiscInfo);
    };

    let path = outdir.join(DISC_INFO_FILENAME);
    disc_info.save(&path)?;

    Ok(disc_info)
}

/// Copy the titles from a disc and save them as MKV files.
///
/// This will run the MakeMKV "mkv" command which will copy all titles that meet the minimum and
/// maximum length requirements to MKV files and save them to the provided output directory. The
/// length requirements are configured via MakeMKV and not adjustable by this program.
///
/// In addition to creating the MKV files, the log output will be written to the output directory
/// in a file called "makemkv.mkv.log".
///
/// This function will take a long time to complete.
///
/// It is expected that the provided output directory will already exist and that the directory
/// won't contain preexisting log or MKV files ("info" command related files are ok). If any of
/// these expectations aren't met, the function will return an error. It is up to the caller to
/// create the directory and delete existing files if required.
pub fn run_copy_cmd<T>(device: &str, outdir: &Path, observer: &mut T) -> Result<()>
where
    T: Observe,
{
    // NOTE: We don't check if the device path is valid/exists here since its likely to be OS
    //       dependent. If it happens to be invalid, the command should fail early. Will revisit if
    //       it becomes an issue in the future.
    //
    if !outdir.is_dir() {
        return Err(Error::OutputDirDoesNotExist(outdir.to_owned()));
    }

    if contains_mkv_files(outdir)? {
        return Err(Error::FoundExistingMkvFiles(outdir.to_path_buf()));
    }

    let mut ctx = Context::new(device, outdir, observer);
    ctx.enable_cmd_log(COPY_CMD_LOG_FILENAME)?;

    commands::run_mkv_command::<OsRunner>(&mut ctx)?;

    Ok(())
}

/// Checks for the existence of MKV files in a directory.
fn contains_mkv_files(dir: &Path) -> Result<bool> {
    for item in dir
        .read_dir()
        .map_err(Error::ExistingMkvFilesCheckIoError)?
    {
        let item = item.map_err(Error::ExistingMkvFilesCheckIoError)?;
        if item.path().extension().and_then(|s| s.to_str()) == Some("mkv") {
            return Ok(true);
        }
    }
    Ok(false)
}
