// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new<P>(file_name: P) -> TempDir
    where
        P: AsRef<Path>
    {
        TempDir(env::temp_dir().join(file_name))
    }

    pub fn path(&self) -> &Path {
        let TempDir(ref p) = *self;
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let TempDir(ref p) = *self;
        if !p.exists() {
            return
        }
        let result = fs::remove_dir_all(p);
        // Avoid panicking while panicking as this causes the process to immediately abort,
        // without displaying test results.
        if !thread::panicking() {
            result.unwrap();
        }
    }
}

pub struct TempFile(pub PathBuf);

impl TempFile {
    pub fn new<P>(file_name: P) -> TempFile
    where
        P: AsRef<Path>
    {
        TempFile(env::temp_dir().join(file_name))
    }

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
        let result = fs::remove_file(p);
        // Avoid panicking while panicking as this causes the process to immediately abort,
        // without displaying test results.
        if !thread::panicking() {
            result.unwrap();
        }
    }
}
