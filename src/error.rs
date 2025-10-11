// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application error and result types.

use std::path::PathBuf;

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the application
#[derive(Debug)]
pub enum Error {
    /// Error raised when a copy service cannot be initialized.
    CopyServiceInit {
        error: copy_srv::Error,
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
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
}
