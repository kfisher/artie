// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

pub struct TempFile(pub PathBuf);

impl TempFile {
    pub fn path(&self) -> &Path {
        let TempFile(ref p) = *self;
        p
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let TempFile(ref p) = *self;
        let result = fs::remove_file(p);
        // Avoid panicking while panicking as this causes the process to immediately abort,
        // without displaying test results.
        if !thread::panicking() {
            result.unwrap();
        }
    }
}
