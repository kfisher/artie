// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Defines the result and error types used by the `makemkv` crate.

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

    /// Error raised when parsing a message and more than one instance of a disc attribute was
    /// reported by MakeMKV.
    DuplicateDiscAttribute(Attribute),

    /// Error raised when parsing a message and more than one instance of a title attribute was
    /// reported by MakeMKV for the same title.
    DuplicateTitleAttribute(Attribute),

    /// Error raised when parsing a message and more than one instance of a stream attribute was
    /// reported by MakeMKV for the same stream.
    DuplicateStreamAttribute(Attribute),

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
}
