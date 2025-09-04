// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Result and error types.

use std::path::PathBuf;

/// Result type for the `handbrake` crate functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for `makemkv` crate functions.
#[derive(Debug)]
pub enum Error {
    /// Error raised when attempting to run a command while it is already running.
    CommandAlreadyRunning,

    /// Error raised when attempting to stop or wait for a command that has not started.
    CommandNotStarted,

    /// Error raised when attempting to start, stop, or wait for a command fails because of an I/O
    /// error.
    CommandIoError {
        error: std::io::Error,
    },

    /// Error raised from the standard error processing thread.
    ErrThreadIoError {
        error: std::io::Error,
    },

    /// Error raised when the standard error thread panics.
    ErrThreadPanicked {
        error: String,
    },

    /// Error raised when the standard error processing thread fails to send on its data channel.
    ErrThreadSendError {
        error: std::sync::mpsc::SendError<crate::command::ChannelData>,
    },

    /// Error raised when attempting to run handbrake with invalid options.
    InvalidOption {
        option: String,
        error: String,
    },

    /// Error raised when attempting to parse JSON data from handbrake output fails.
    JsonParseError {
        error: serde_json::Error,
    },

    /// Error raised when the file for logging command output already exists or the existance of
    /// the file could not be verified one way or the other due to an error.
    LogFileExists {
        path: PathBuf,
        error: Option<std::io::Error>,
    },

    /// Error raised when the file for logging command output fails to open.
    LogFileOpenError {
        path: PathBuf,
        error: std::io::Error,
    },

    /// Error raised when logging a line of standard output text fails.
    LogStdOutError {
        path: PathBuf,
        error: std::io::Error,
    },

    /// Error raised when logging a line of standard error text fails.
    LogStdErrError {
        path: PathBuf,
        error: std::io::Error,
    },

    /// Error raised from the standard output processing thread.
    OutThreadIoError {
        error: std::io::Error,
    },

    /// Error raised when the standard output thread panics.
    OutThreadPanicked {
        error: String,
    },

    /// Error raised when the standard output processing thread fails to send on its data channel.
    OutThreadSendError {
        error: std::sync::mpsc::SendError<crate::command::ChannelData>,
    },

    /// Error raised when attempting to parse JSON data from handbrake output fails because of an
    /// I/O error when managing the text buffer.
    ParseOutputIoError {
        error: std::io::Error,
    },

    /// Error raised when attempting to add text to the progress parser's internal buffer.
    ProgressBufferWriteError {
        text: String,
        error: std::io::Error,
    },

    /// Error raised when parsing output from the output console encounters something unexpected.
    UnexpectedOutput {
        text: String,
    }
}
