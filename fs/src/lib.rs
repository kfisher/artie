// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for file system operations.
//!
//! This crate is used to perform file system operations related to managing media and data files
//! that are generated when performing copy and transcode operations. 

pub mod error;

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Container for root directory paths.
#[derive(Debug)]
pub struct Folder {
    /// Path to the media inbox directory.
    ///
    /// The inbox directory is where the titles are initially saved to when being copyied from a 
    /// disc along with some additional data files.
    pub inbox: PathBuf,

    /// Path to the media library directory.
    ///
    /// This is where the videos are stored for use by the streaming service.
    pub library: PathBuf,

    /// Path to the archive directory.
    ///
    /// This is where videos not moved to the library will be moved to.
    pub archive: PathBuf,

    /// Path to the directory where application data should be stored.
    pub data: PathBuf,
}

impl Folder {
    /// Creates a [`Folder`] instance.
    pub fn new(inbox: &Path, library: &Path, archive: &Path, data: &Path) -> Self {
        Self {
            inbox: inbox.to_owned(),
            library: library.to_owned(),
            archive: archive.to_owned(),
            data: data.to_owned(),
        }
    }

    /// Creates the folders if they don't already exist.
    pub fn make_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.inbox)
            .map_err(|error| Error::CreateDirectory {
                path: self.inbox.to_owned(),
                error 
            })?;

        fs::create_dir_all(&self.library)
            .map_err(|error| Error::CreateDirectory {
                path: self.library.to_owned(),
                error 
            })?;

        fs::create_dir_all(&self.archive)
            .map_err(|error| Error::CreateDirectory {
                path: self.archive.to_owned(),
                error 
            })?;

        fs::create_dir_all(&self.data)
            .map_err(|error| Error::CreateDirectory {
                path: self.data.to_owned(),
                error 
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    use std::thread;

    pub struct TempFile(pub PathBuf);

    impl TempFile {
        fn new(file_name: &str) -> TempFile {
            TempFile(env::temp_dir().join(file_name))
        }

        fn path(&self) -> &Path {
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
                fs::remove_dir_all(p)
            } else {
                fs::remove_file(p)
            };
            // Avoid panicking while panicking as this causes the process to immediately abort,
            // without displaying test results.
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }

    #[test]
    fn test_make_dirs_success() {
        let temp = TempFile::new("artie.test.fs.make_dirs_success");
        let base = temp.path();

        let folders = Folder {
            inbox: base.join("inbox"),
            library: base.join("library"),
            archive: base.join("archive"),
            data: base.join("data"),
        };

        // Create the directories
        let result = folders.make_dirs();
        assert!(result.is_ok());

        // Verify all directories exist
        assert!(folders.inbox.exists());
        assert!(folders.library.exists());
        assert!(folders.archive.exists());
        assert!(folders.data.exists());
    }

    #[test]
    fn test_make_dirs_with_nested_paths() {
        let temp = TempFile::new("artie.test.fs.make_dirs_with_nested_paths");
        let base = temp.path();

        let folders = Folder {
            inbox: base.join("parent/child/inbox"),
            library: base.join("parent/child/library"),
            archive: base.join("parent/child/archive"),
            data: base.join("parent/child/data"),
        };

        let result = folders.make_dirs();
        assert!(result.is_ok());

        // Verify all directories exist, including parent directories
        assert!(folders.inbox.exists());
        assert!(folders.library.exists());
        assert!(folders.archive.exists());
        assert!(folders.data.exists());
    }

    #[test]
    fn test_make_dirs_already_exist() {
        let temp = TempFile::new("artie.test.fs.make_dirs_already_exists");
        let base = temp.path();

        let folders = Folder {
            inbox: base.join("inbox"),
            library: base.join("library"),
            archive: base.join("archive"),
            data: base.join("data"),
        };

        // Create directories first time
        folders.make_dirs().unwrap();

        // Create directories second time (should succeed without error)
        let result = folders.make_dirs();
        assert!(result.is_ok());

        // Verify all directories still exist
        assert!(folders.inbox.exists());
        assert!(folders.library.exists());
        assert!(folders.archive.exists());
        assert!(folders.data.exists());
    }

    #[test]
    fn test_make_dirs_are_actually_directories() {
        let temp = TempFile::new("artie.test.fs.make_dirs_actually_directories");
        let base = temp.path();

        let folders = Folder {
            inbox: base.join("inbox"),
            library: base.join("library"),
            archive: base.join("archive"),
            data: base.join("data"),
        };

        folders.make_dirs().unwrap();

        // Verify they are directories, not files
        assert!(folders.inbox.is_dir());
        assert!(folders.library.is_dir());
        assert!(folders.archive.is_dir());
        assert!(folders.data.is_dir());
    }

    #[test]
    fn test_make_dirs_with_file_conflict() {
        let temp = TempFile::new("artie.test.fs.make_dirs_with_file_conflict");
        let base = temp.path();

        let file_path = base.join("inbox");
        
        // Create a file where a directory should be
        fs::create_dir_all(base).unwrap();
        fs::write(&file_path, "test").unwrap();

        let folders = Folder {
            inbox: file_path,
            library: base.join("library"),
            archive: base.join("archive"),
            data: base.join("data"),
        };

        // This should fail because inbox is a file, not a directory
        let result = folders.make_dirs();
        assert!(result.is_err());
    }
}

