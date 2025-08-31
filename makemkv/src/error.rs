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

    // NOTE: Even though `InvalidMessageFormat` has a single value, use a structured variant to
    //       avoid ambiguity (might be easy to mistake for an error message).
    /// Error raised when parsing a message and the message cannot be parsed into its key/value
    /// components.
    InvalidMessageFormat {
        /// The raw message text that could not be parsed.
        msg: String,
    },

    /// Error raised when parsing a message and the message type is unknown to the parser and
    /// cannot parsed.
    UnknownMessageType {
        /// Key component of the message which identifies the message type.
        key: String,

        /// Data component of the message.
        data: String,
    },

    /// Error raised when parsing a message and more than one instance of a disc, title, or stream
    /// attribute per instance was reported by MakeMKV.
    DuplicateAttribute(Attribute),

    /// Error raised within the thread processing the output from a running MakeMKV command failed
    /// to read the of output due to an I/O error.
    CommandOutThreadIoError(std::io::Error),

    /// Error raised within the thread processing the output from a running MakeMKV command failed
    /// to send data on its out channel.
    ///
    /// This should only be possible if the receiving end of the channel was closed. Refer to the
    /// documentation for `std::sync::mpsc::SendError` for more information.
    CommandOutThreadSendError,

    /// Error raised within the thread processing the output from a running MakeMKV command
    /// panicked.
    CommandOutThreadPanicked,

    /// Error raised within the thread processing the error output from a running MakeMKV command
    /// failed to read the of output due to an I/O error.
    CommandErrThreadIoError(std::io::Error),

    /// Error raised within the thread processing the error output from a running MakeMKV command
    /// failed to send data on its out channel.
    ///
    /// This should only be possible if the receiving end of the channel was closed. Refer to the
    /// documentation for `std::sync::mpsc::SendError` for more information.
    CommandErrThreadSendError,

    /// Error raised within the thread processing the error output from a running MakeMKV command
    /// panicked.
    CommandErrThreadPanicked,

    /// Error raised when the MakeMKV failed to start because it was already running.
    CommandAlreadyRunning,

    /// Error raised when the MakeMKV failed to start due to an I/O error.
    CommandStartIoError(std::io::Error),

    /// Error raised when the request to wait for the MakeMKV command to complete fails because the
    /// command is not in a state where it can be waited on.
    CommandWaitInvalidStateError,

    /// Error raised when the request to wait for the MakeMKV command to complete fails due to an
    /// I/O error.
    CommandWaitIoError(std::io::Error),

    /// Error raised when the request to stop the MakeMKV command fails because the command is not
    /// in a state where it can be stopped.
    CommandKillInvalidStateError,

    /// Error raised when the request to stop the MakeMKV command fails due to an I/O error.
    CommandKillIoError(std::io::Error),

    /// Error raised by the output processor when its unable to process a message.
    ProcessMessageError,

    /// Error raised by the output processor when its unable to process error output.
    ProcessErrorOutputError,

    /// Error raised when the output directory does not exist or is not a directory.
    OutputDirDoesNotExist(PathBuf),

    /// Error raised when the attempting to open a file and it fails.
    FileOpenError {
        /// Path to the file that failed to open.
        path: PathBuf,

        /// The I/O error that that caused the failure.
        error: std::io::Error,
    },

    /// Error raised when the JSON file the disc information will be written to already exists.
    DiscInfoFileExists(PathBuf),

    /// Error raised when checking for existing MKV files.
    ExistingMkvFilesCheckIoError(std::io::Error),

    /// Error raised when one or more MKV files already exist in the provided output directory.
    FoundExistingMkvFiles(PathBuf),

    /// Error raised when the info command completed successfully, but failed to read any disc
    /// disc information.
    MissingDiscInfo,

    /// Error raised when attempting to write raw command output to a log file.
    CommandLogError(std::io::Error),

    /// Error raised when trying to serialize or deserialize JSON data.
    JsonError {
        /// Path to the file the JSON was being written to or read from.
        path: PathBuf,

        /// The error that was raised.
        error: serde_json::Error,
    },
}
