// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application error and result types.

use std::path::PathBuf;

/// Error type for the application
#[derive(Debug)]
pub enum Error {
    /// Error raised when a command (external subprocess) fails.
    CommandIo {
        command: String,
        args: String,
        error: std::io::Error,
    },

    /// Indicates an external command existed with an error code.
    CommandReturnedErrorCode {
        command: String,
        args: String,
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },

    /// Error raised when connecting to the database fails.
    Connect {
        path: Option<PathBuf>,
        error: rusqlite::Error,
    },

    /// An error that can occur when converting raw bytes to a string, or vice-versa.
    ConversionError {
        error: std::string::FromUtf8Error,
    },

    /// Error raised when a copy service cannot be initialized.
    CopyServiceInit {
        error: crate::copy_srv::Error,
    },

    /// Error raised when performing a database operation.
    Db {
        operation: crate::db::Operation,
        error: rusqlite::Error,
    },

    /// Error raised when a string argument provided to a database operation is empty when the 
    /// operation expects a non-empty string.
    ///
    /// TODO: Should convert this to a generic DB validation error.
    EmptyString {
        arg: String,
    },

    /// Error raised when attempting to read or write to a file fails.
    FileIo {
        path: PathBuf,
        error: std::io::Error,
    },

    /// Error raised when a file cannot be found.
    ///
    /// This may also be raised if the path is not a file or if the user does not have the required
    /// permissions to know of the file's existence.
    FileNotFound {
        path: PathBuf,
    },

    /// Error raised when serializing or deserializing data.
    ///
    /// If `path` is `Some`, its the path to the file the serialized data was read from or about to 
    /// be written to for additional context.
    Serialization {
        path: Option<PathBuf>,
        error: SerializationError,
    },
}

/// Error subtype to encapsulate various serialization errors.
#[derive(Debug)]
pub enum SerializationError {
    JsonDeserialize(serde_json::Error),
    JsonSerialize(serde_json::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
}

/// Creates a [`Error::CommandIo`] error based on the provided command and error.
pub fn command_io(command: &std::process::Command, error: std::io::Error) -> Error {
    Error::CommandIo { 
        command: command
            .get_program()
            .to_string_lossy()
            .into_owned(),
        args: command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        error,
    }
}

/// Creates a [`Error::CommandReturnedErrorCode`] error based on the provided command and output.
pub fn command_exit(command: &std::process::Command, output: &std::process::Output) -> Error {
    Error::CommandReturnedErrorCode {
        command: command
            .get_program()
            .to_string_lossy()
            .into_owned(),
        args: command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        code: output.status.code(),
        stdout: String::from_utf8_lossy(output.stdout.as_slice())
            .into_owned(),
        stderr: String::from_utf8_lossy(output.stderr.as_slice())
            .into_owned(),
    }
}

/// Creates a [`Error::Serialization`] error when caused by failing to parse JSON.
pub fn json_deserialize(error: serde_json::Error) -> Error {
    Error::Serialization {
        path: None,
        error: SerializationError::JsonDeserialize(error),
    }
}


