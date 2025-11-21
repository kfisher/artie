// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides general file system utilities.

use std::env;
use std::path::{Path, PathBuf};
use std::thread;

/// Generates a path to a temporary directory or file that will automatically get deleted when
/// the struct instance is dropped.
///
/// By default, this will not create the file or directory. If the struct instance is dropped prior
/// to the file or directory being created, it will safely exit without attempting to delete.
pub struct TempFile(pub PathBuf);

impl TempFile {
    /// Creates a new temporary file path to a directory with the provided name.
    ///
    /// This essentially just appends the provided name to the operating system's temporary file 
    /// location.
    pub fn new(file_name: &str) -> TempFile {
        TempFile(env::temp_dir().join(file_name))
    }

    /// Returns the path of the temporary file or directory.
    pub fn path(&self) -> &Path {
        let TempFile(ref p) = *self;
        p
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let TempFile(ref p) = *self;
        if !p.exists() {
            return
        }
        let result = if p.is_dir() {
            std::fs::remove_dir_all(p)
        } else {
            std::fs::remove_file(p)
        };
        // Avoid panicking while panicking as this causes the process to immediately abort,
        // without displaying test results.
        if !thread::panicking() {
            result.unwrap();
        }
    }
}

// TODO: Should there be a test? So far, this is only used in testing code anyways.
