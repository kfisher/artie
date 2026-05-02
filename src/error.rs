// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application errors.

use std::path::PathBuf;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::bus;
use crate::db;
use crate::drive;
use crate::models::MediaLocation;
use crate::net;

/// Specifies the errors that can occur throughout the application.
#[derive(Debug)]
pub enum Error {
    /// Raised when attempting to cancel an operation fails because the cancellation token is not
    /// available.
    CancelTokenNone,

    /// Raised when attempting to send a message to an actor fails.
    ChannelSend(ChannelSendError),

    /// Indicates an external command existed with an error code.
    CommandReturnedErrorCode {
        command: String,
        args: String,
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },

    /// Raised by the GTK library.
    Gtk(gtk::glib::Error),

    /// Raised when database operations fail.
    Database(rusqlite::Error),

    /// Raised when attempting to send a message to the control or worker node when they are not
    /// connected.
    Disconnected,

    /// Raised when a drive request fails because a drive actor associated with the specified
    /// serial number could not be found.
    DriveNotFound {
        serial_number: String,
    },

    /// Raised when a file cannot be found.
    ///
    /// This may also be raised if the path is not a file or if the user does not have the required
    /// permissions to know of the file's existence.
    FileNotFound {
        path: PathBuf,
    },

    /// Raised when a drive actor gets a request meant for the manager or the request serial number
    /// does not match its associated drive serial number.
    InvalidDriveRequest,

    /// Raised when attempting to perform a drive action that cannot be done in the current state.
    InvalidDriveState {
        state: String,
    },

    /// Error raise when attempting to use an invalid media location.
    ///
    /// This will typically be raised if attempting to use [`MediaLocation::Deleted`] when a valid
    /// path is expected.
    InvalidMediaLocation {
        location: MediaLocation,
    },

    /// Raised when a MakeMKV command fails.
    MakeMkv(makemkv::Error),

    /// Raised when an audio codec mapping cannot be found.
    ///
    /// Will be raised when looking up the MakeMKV audio codec and there is not a mapping for the
    /// provided codec (short form).
    MissingAudioCodecMapping {
        codec_short: String,
    },

    /// Raised when an subtitle codec mapping cannot be found.
    ///
    /// Will be raised when looking up the MakeMKV subtitle codec and there is not a mapping for
    /// the provided codec (short form).
    MissingSubtitleCodecMapping {
        codec_short: String,
    },

    /// Raised when an video codec mapping cannot be found.
    ///
    /// Will be raised when looking up the MakeMKV video codec and there is not a mapping for the
    /// provided codec (short form).
    MissingVideoCodecMapping {
        codec_short: String,
    },

    /// Raised when a message cannot be sent to or from the task responsible for handling network
    /// communication.
    NetworkChannelSend(NetworkChannelSendError),

    /// Raised as a response to a send request when the message cannot be sent over the network.
    ///
    /// This will be the error sent to the requester as the response to the request. The true cause
    /// will be logged prior to sending the response.
    NetworkSend,

    /// Raised when attempting to receive a response to a message.
    ResponseRecv(oneshot::error::RecvError),

    /// Raised when attempting to send a response fails.
    ResponseSend,

    /// Raised when serializing or deserializing JSON fails.
    SerdeJson(serde_json::Error),

    /// Raised when an error occurs while performing I/O operations.
    StdIo(std::io::Error),

    /// Raised when deserializing TOML.
    TomlDeserialize(toml::de::Error),

    /// Raised when serializing TOML.
    TomlSerialize(toml::ser::Error),

    /// Error raised when an unexpected stream type is encountered.
    UnexpectedStreamType {
        stream_type: Option<String>,
    },

    /// Error raised when an unexpected file extension is encountered.
    UnexpectedFileExtension {
        expected: String,
        actual: String,
    },

    /// Error raised when an actor gets a request that it does not support.
    ///
    /// This will mainly be seen on actors where requests are handled differently on the control
    /// node vs the worker node.
    UnsupportedRequest {
        request: String,
    },

    /// Raised when an array of bytes cannot be converted into a UTF-8 string.
    UtfConversion(std::string::FromUtf8Error),

    /// Raised when attempting to use an invalid argument.
    ///
    /// In general, when this error is raised, the validation error should have been detected
    /// before hand.
    Validation {
        error: ValidationError,
        arg: String,
    }
}

impl From<gtk::glib::Error> for Error {
    fn from(value: gtk::glib::Error) -> Self {
        Error::Gtk(value)
    }
}

impl From<makemkv::Error> for Error {
    fn from(value: makemkv::Error) -> Self {
        Error::MakeMkv(value)
    }
}

impl From<mpsc::error::SendError<bus::Message>> for Error {
    fn from(value: mpsc::error::SendError<bus::Message>) -> Self {
        Error::ChannelSend(ChannelSendError::MessageBus(value))
    }
}

impl From<mpsc::error::SendError<db::Message>> for Error {
    fn from(value: mpsc::error::SendError<db::Message>) -> Self {
        Error::ChannelSend(ChannelSendError::Database(value))
    }
}

impl From<mpsc::error::SendError<drive::Message>> for Error {
    fn from(value: mpsc::error::SendError<drive::Message>) -> Self {
        Error::ChannelSend(ChannelSendError::Drive(value))
    }
}

impl From<mpsc::error::SendError<net::Message>> for Error {
    fn from(value: mpsc::error::SendError<net::Message>) -> Self {
        Error::ChannelSend(ChannelSendError::Net(value))
    }
}

impl From<mpsc::error::SendError<net::IncomingMessage>> for Error {
    fn from(value: mpsc::error::SendError<net::IncomingMessage>) -> Self {
        Error::NetworkChannelSend(NetworkChannelSendError::Incoming(value))
    }
}

impl From<mpsc::error::SendError<net::OutgoingMessage>> for Error {
    fn from(value: mpsc::error::SendError<net::OutgoingMessage>) -> Self {
        Error::NetworkChannelSend(NetworkChannelSendError::Outgoing(value))
    }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(value: oneshot::error::RecvError) -> Self {
        Error::ResponseRecv(value)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Error::Database(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::StdIo(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Error::UtfConversion(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::TomlDeserialize(value)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::TomlSerialize(value)
    }
}

/// Specifies the errors that can occur when attempting to send a message.
#[derive(Debug)]
pub enum ChannelSendError {
    /// Error raised when sending a message to the database actor fails.
    Database(mpsc::error::SendError<db::Message>),

    /// Error raised when sending a message to the drive manager or a drive actor fails.
    Drive(mpsc::error::SendError<drive::Message>),

    /// Error raised when sending a message to the message bus fails.
    MessageBus(mpsc::error::SendError<bus::Message>),

    /// Error raised when sending a message to the client or server fails.
    Net(mpsc::error::SendError<net::Message>),
}

/// Specifies the errors that can occur when attempting to send a message message to or from the
/// task handing network communication.
#[derive(Debug)]
pub enum NetworkChannelSendError {
    Incoming(mpsc::error::SendError<net::IncomingMessage>),
    Outgoing(mpsc::error::SendError<net::OutgoingMessage>),
}

/// Specifies the errors that can occur when attempting to invalid arguments.
#[derive(Debug)]
pub enum ValidationError {
    EmptyString,
}

#[cfg(test)]
mod tests {
    // TODO
}

