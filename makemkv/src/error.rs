// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Result and error types.

use std::path::PathBuf;

use crate::data::Attribute;

/// Result type for `makemkv` crate functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for `makemkv` crate functions.
#[derive(Debug)]
pub enum Error {
    /// Error raised when the MakeMKV failed to start because it was already running.
    CommandAlreadyRunning,

    /// Error raised when attempting to stop or wait for a command that has not started.
    CommandNotStarted,

    /// Error raised when attempting to start, stop, or wait for a command fails because of an I/O
    /// error.
    CommandIoError {
        error: std::io::Error,
    },

    /// Error raised when parsing a message and more than one instance of a disc, title, or stream
    /// attribute per instance was reported by MakeMKV.
    DuplicateAttribute {
        attr: Attribute
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
        error: std::sync::mpsc::SendError<crate::commands::ChannelData>,
    },

    /// Error raised when checking for existing MKV files.
    ExistingMkvFilesCheckIoError {
        error: std::io::Error
    },

    /// Error raised when the attempting to open a file and it fails.
    FileOpenError {
        path: PathBuf,
        error: std::io::Error,
    },

    /// Error raised when one or more MKV files already exist in the provided output directory.
    FoundExistingMkvFiles {
        path: PathBuf
    },

    /// Error raised when parsing a message from MakeMKV when the data within the message cannot be
    /// parsed because the data is malformed or missing.
    InvalidMessageData {
        /// Key component of the message which identifies the message type.
        key: String,

        /// Data component of the message which could not be parsed.
        data: String,

        /// Error is a brief description of the error.
        error: String,
    },

    /// Error raised when parsing a message and the message cannot be parsed into its key/value
    /// components.
    InvalidMessageFormat {
        /// The raw message text that could not be parsed.
        msg: String,
    },

    /// Error raised when trying to serialize or deserialize JSON data.
    JsonError {
        path: PathBuf,
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

    /// Error raised when the info command completed successfully, but failed to read any disc
    /// disc information.
    MissingDiscInfo,

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
        error: std::sync::mpsc::SendError<crate::commands::ChannelData>,
    },

    /// Error raised when parsing a message and the message type is unknown to the parser and
    /// cannot parsed.
    UnknownMessageType {
        key: String,
        data: String,
    },
}
