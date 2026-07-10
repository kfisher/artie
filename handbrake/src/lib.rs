// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for running the HandBrake command.

mod command;
mod error;
mod output;

pub use crate::error::{Error, Result};
pub use crate::command::{AudioTrackOption, Options};

use std::path::Path;
use std::process::ExitStatus;
use std::sync::OnceLock;

use crate::command::{Context};

/// Trait for recieving status updates from a running HandBrake command.
pub trait Observe {
    /// Called when a general log message it output by HandBrake.
    fn message(&mut self, _msg: &str) {}

    /// Called when progress data is received from HandBrake.
    fn progress(&mut self, _progress: Progress) {}

    /// Called when version data is received from HandBrake.
    fn version(&mut self, _version: Version) {}
}

/// Represents the progress of a running HandBrake command.
#[derive(Default)]
pub struct Progress {
    /// The current transcode pass number.
    pub pass: i32,

    /// The total number of transcode passes that will be performed.
    pub pass_count: i32,

    /// The current percent complete of the current pass.
    pub progress: i32,
}

/// Represents version information about HandBrake.
#[derive(Default)]
pub struct Version {
    /// The system architecture.
    pub arch: String,

    /// The system type.
    pub system: String,

    /// The handbrake version.
    pub version: String,
}

/// Returns the list of available HandBrake presets.
pub fn presets() -> &'static Vec<&'static str> {
    // TODO: Instead of having a hard-coded list, should initialize by running the HandBrake
    //       command to get the list of presets.
    static PRESET_LIST: OnceLock<Vec<&'static str>> = OnceLock::new();
    PRESET_LIST.get_or_init(|| {
        vec![
            "Very Fast 1080p30",
            "Fast 1080p30",
            "HQ 1080p30 Surround",
            "Super HQ 1080p30 Surround",
        ]
    })
}

/// Run handbrake to transcode a video.
///
/// <div class="warning">
///
/// Calling this function will block until the command completes. This can take a long time
/// depending on the options and the system its being run on.
///
/// </div>
///
/// `opts` are the configurable options for the transcode. At a minimum, this requires the
/// HandBrake preset to use, the input video file path, and where to save the transcoded video.
///
/// `observer` provides the ability for the caller to receive callbacks while the handbrake is
/// running with information messages and the current progress.
///
/// `log_path` path to where to save the raw output from HandBrake.
///
/// On success, returns the exit code of the handbrake command.
pub fn transcode_video<T>(opts: &Options, observer: &mut T, log_path: &Path) -> Result<ExitStatus>
where
    T: Observe
{
    let mut ctx = Context::new(observer);
    ctx.log_output(log_path)?;

    command::run_handbrake(&mut ctx, opts)
}

/// Returns the list of available HandBrake video encoders.
pub fn video_encoders() -> &'static Vec<&'static str> {
    // TODO: Instead of having a hard-coded list, should initialize by running the HandBrake
    //       command to get the list of encorders.
    static PRESET_LIST: OnceLock<Vec<&'static str>> = OnceLock::new();
    PRESET_LIST.get_or_init(|| {
        vec![
            "H.264 (x264)",
            "H.264 (NVEnc)",
            "H.265 (x265)",
            "H.265 (NVEnc)",
        ]
    })
}
