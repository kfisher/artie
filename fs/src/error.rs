// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Result and error types.

use std::path::PathBuf;

/// Result type for `fs` crate functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the `fs` crate.
#[derive(Debug)]
pub enum Error {
    /// Error raised when a directory could not be created.
    CreateDirectory {
        path: PathBuf,
        error: std::io::Error,
    },
}
