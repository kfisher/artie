// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Crate responsible for running the MakeMKV command.

mod commands;
mod data;
mod error;
mod messages;

pub use crate::data::DiscInfo;
pub use crate::error::{Error, Result};

use std::fs;
use std::path::Path;

use crate::commands::{Context, OsRunner};

/// Filename where the "info" command log is written to.
pub const INFO_CMD_LOG_FILENAME: &str = "makemkv.info.log";

/// Filename where the "mkv" command log is written to.
pub const COPY_CMD_LOG_FILENAME: &str = "makemkv.mkv.log";

/// Filename where the disc information is written to.
pub const DISC_INFO_FILENAME: &str = "disc_info.json";


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
pub fn run_info_cmd(device: &str, outdir: &Path) -> Result<DiscInfo> {
    //
    // NOTE: We don't check if the device path is valid/exists here since its likely to be OS
    //       dependent. If it happens to be invalid, the command should fail early. Will revisit if
    //       it becomes an issue in the future.
    //
    if !outdir.is_dir() {
        return Err(Error::OutputDirDoesNotExist(outdir.to_owned()));
    }

    let mut ctx = Context::new(device, outdir);

    let path = ctx.info_log_path();
    if path.exists() {
        return Err(Error::LogFileExists(path));
    }

    let path = ctx.disc_info_path();
    if path.exists() {
        return Err(Error::DiscInfoFileExists(path));
    }

    commands::run_info_command::<OsRunner>(&mut ctx)?;

    match ctx.disc_info {
        Some(disc_info) => Ok(disc_info),
        None => Err(Error::MissingDiscInfo),
    }
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
pub fn run_copy_cmd(device: &str, outdir: &Path) -> Result<()> {
    //
    // NOTE: We don't check if the device path is valid/exists here since its likely to be OS
    //       dependent. If it happens to be invalid, the command should fail early. Will revisit if
    //       it becomes an issue in the future.
    //
    if !outdir.is_dir() {
        return Err(Error::OutputDirDoesNotExist(outdir.to_owned()));
    }

    let mut ctx = Context::new(device, outdir);

    let path = ctx.info_log_path();
    if path.exists() {
        return Err(Error::LogFileExists(path));
    }

    if contains_mkv_files(outdir)? {
        return Err(Error::FoundExistingMkvFiles(path));
    }

    commands::run_mkv_command::<OsRunner>(&mut ctx)?;

    Ok(())
}

/// Checks for the existence of MKV files in a directory.
fn contains_mkv_files(dir: &Path) -> Result<bool> {
    for item in dir.read_dir().map_err(Error::ExistingMkvFilesCheckIoError)? {
        let item = item.map_err(Error::ExistingMkvFilesCheckIoError)?;
        if item.path().extension().and_then(|s| s.to_str()) == Some("mkv") {
            return Ok(true);
        }
    }
    Ok(false)
}

