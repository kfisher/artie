// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Defines the result and error types used by the `makemkv` crate.

/// Result type for `makemkv` crate functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for `makemkv` crate functions.
pub enum Error {
    /// Error raised when parsing a message from MakeMKV when the data within the message cannot be
    /// parsed because the data is malformed or missing.
    InvalidMessageData {
        /// Key component of the message which identifies the message type.
        key: String,

        /// Data component of the message which could not be parsed.
        data: String,

        /// Error is a brief description of the error.
        error: String
    },

    /// Error raised when parsing a message and the message cannot be parsed into its key/value
    /// components.
    InvalidMessageFormat {
        /// The raw message text that could not be parsed.
        msg: String 
    },

    /// Error raised when parsing a message and the message type is unknown to the parser and
    /// cannot parsed.
    UnknownMessageType {
        /// Key component of the message which identifies the message type.
        key: String,

        /// Data component of the message.
        data: String
    },
}

