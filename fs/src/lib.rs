// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for file system operations.
//!
//! This crate is used to perform file system operations related to managing media and data files
//! that are generated when performing copy and transcode operations. 

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use model::{CopyOperation, MediaLocation};

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

/// Container for root directory paths.
#[derive(Clone, Debug)]
pub struct FileSystem {
    settings: Settings,
}

impl FileSystem {
    /// Creates a [`FileSystem`] instance.
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
        }
    }

    /// Returns `true` if the archive directory exists, is a directory, and accessible by the user
    /// or `false` otherwise.
    pub fn archive_exists(&self) -> bool {
        self.settings.archive.is_dir()
    }

    /// Returns `true` if the data directory exists, is a directory, and accessible by the user or
    /// `false` otherwise.
    pub fn data_exists(&self) -> bool {
        self.settings.data.is_dir()
    }

    /// Returns `true` if the inbox directory exists, is a directory, and accessible by the user or
    /// `false` otherwise.
    pub fn inbox_exists(&self) -> bool {
        self.settings.inbox.is_dir()
    }

    /// Returns the path to the inbox folder for a copy operation.
    pub fn inbox_folder(&self, copy_operation: &CopyOperation) -> PathBuf {
        self.settings.inbox.join(match copy_operation.media_type {
            model::MediaType::Movie => {
                PathBuf::from(format!(
                    "0x{:08X}.{}.D{}",
                    copy_operation.id,
                    copy_operation.title,
                    copy_operation.disc
                ))
            },
            model::MediaType::Show => {
                PathBuf::from(format!(
                    "0x{:08X}.{}.S{}.D{}",
                    copy_operation.id,
                    copy_operation.title,
                    copy_operation.season,
                    copy_operation.disc
                ))
            },
        })
    }

    /// Returns `true` if the library directory exists, is a directory, and accessible by the user
    /// or `false` otherwise.
    pub fn library_exists(&self) -> bool {
        self.settings.library.is_dir()
    }

    /// Creates the folders if they don't already exist.
    pub fn make_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.settings.inbox)
            .map_err(|error| Error::CreateDirectory {
                path: self.settings.inbox.to_owned(),
                error 
            })?;

        fs::create_dir_all(&self.settings.library)
            .map_err(|error| Error::CreateDirectory {
                path: self.settings.library.to_owned(),
                error 
            })?;

        fs::create_dir_all(&self.settings.archive)
            .map_err(|error| Error::CreateDirectory {
                path: self.settings.archive.to_owned(),
                error 
            })?;

        fs::create_dir_all(&self.settings.data)
            .map_err(|error| Error::CreateDirectory {
                path: self.settings.data.to_owned(),
                error 
            })?;

        Ok(())
    }

    /// Returns the path for a file.
    pub fn path(&self, location: &MediaLocation) -> Option<PathBuf> {
        match location {
            MediaLocation::Inbox(path) => {
                Some(self.settings.inbox.join(path))
            },
            MediaLocation::Library(path) => {
                Some(self.settings.library.join(path))
            },
            MediaLocation::Archive(path) => {
                Some(self.settings.archive.join(path))
            },
            MediaLocation::Deleted => None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::{Path, PathBuf};
    use std::thread;

    use model::MediaType;

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

    fn setup_file_system(temp: &TempFile, make_dirs: bool) -> FileSystem {
        let base = temp.path();

        let settings = Settings {
            inbox: base.join("inbox"),
            library: base.join("library"),
            archive: base.join("archive"),
            data: base.join("data"),
        };

        let file_system = FileSystem::new(settings);

        if make_dirs {
            let result = file_system.make_dirs();
            assert!(result.is_ok());
        }

        file_system
    }

    #[test]
    fn test_archive_exists() {
        let temp = TempFile::new("artie.test.fs.archive_exists");
        let file_system = setup_file_system(&temp, false);

        assert!(!file_system.archive_exists());
        fs::create_dir_all(&file_system.settings.archive).unwrap();
        assert!(file_system.archive_exists());
    }

    #[test]
    fn test_data_exists() {
        let temp = TempFile::new("artie.test.fs.data_exists");
        let file_system = setup_file_system(&temp, false);

        assert!(!file_system.data_exists());
        fs::create_dir_all(&file_system.settings.data).unwrap();
        assert!(file_system.data_exists());
    }

    #[test]
    fn test_inbox_exists() {
        let temp = TempFile::new("artie.test.fs.inbox_exists");
        let file_system = setup_file_system(&temp, false);

        assert!(!file_system.inbox_exists());
        fs::create_dir_all(&file_system.settings.inbox).unwrap();
        assert!(file_system.inbox_exists());
    }

    #[test]
    fn test_library_exists() {
        let temp = TempFile::new("artie.test.fs.library_exists");
        let file_system = setup_file_system(&temp, false);

        assert!(!file_system.library_exists());
        fs::create_dir_all(&file_system.settings.library).unwrap();
        assert!(file_system.library_exists());
    }

    #[test]
    fn test_make_dirs_success() {
        let temp = TempFile::new("artie.test.fs.make_dirs_success");
        let file_system = setup_file_system(&temp, false);

        // Create the directories
        let result = file_system.make_dirs();
        assert!(result.is_ok());

        // Verify all directories exist
        assert!(file_system.settings.inbox.exists());
        assert!(file_system.settings.library.exists());
        assert!(file_system.settings.archive.exists());
        assert!(file_system.settings.data.exists());
    }

    #[test]
    fn test_make_dirs_with_nested_paths() {
        let temp = TempFile::new("artie.test.fs.make_dirs_with_nested_paths");
        let file_system = setup_file_system(&temp, false);

        let result = file_system.make_dirs();
        assert!(result.is_ok());

        // Verify all directories exist, including parent directories
        assert!(file_system.settings.inbox.exists());
        assert!(file_system.settings.library.exists());
        assert!(file_system.settings.archive.exists());
        assert!(file_system.settings.data.exists());
    }

    #[test]
    fn test_make_dirs_already_exist() {
        let temp = TempFile::new("artie.test.fs.make_dirs_already_exists");
        let file_system = setup_file_system(&temp, false);

        // Create directories first time
        file_system.make_dirs().unwrap();

        // Create directories second time (should succeed without error)
        let result = file_system.make_dirs();
        assert!(result.is_ok());

        // Verify all directories still exist
        assert!(file_system.settings.inbox.exists());
        assert!(file_system.settings.library.exists());
        assert!(file_system.settings.archive.exists());
        assert!(file_system.settings.data.exists());
    }

    #[test]
    fn test_make_dirs_are_actually_directories() {
        let temp = TempFile::new("artie.test.fs.make_dirs_actually_directories");
        let file_system = setup_file_system(&temp, false);

        file_system.make_dirs().unwrap();

        // Verify they are directories, not files
        assert!(file_system.settings.inbox.is_dir());
        assert!(file_system.settings.library.is_dir());
        assert!(file_system.settings.archive.is_dir());
        assert!(file_system.settings.data.is_dir());
    }

    #[test]
    fn test_make_dirs_with_file_conflict() {
        let temp = TempFile::new("artie.test.fs.make_dirs_with_file_conflict");
        let base = temp.path();

        let file_path = base.join("inbox");

        // Create a file where a directory should be
        fs::create_dir_all(base).unwrap();
        fs::write(&file_path, "test").unwrap();

        let settings = Settings {
            inbox: file_path,
            library: base.join("library"),
            archive: base.join("archive"),
            data: base.join("data"),
        };

        let file_system = FileSystem::new(settings);

        // This should fail because inbox is a file, not a directory
        let result = file_system.make_dirs();
        assert!(result.is_err());
    }

    #[test]
    fn test_movie_inbox_folder() {
        let temp = TempFile::new("artie.test.fs.test_movie_inbox_folder");
        let file_system = setup_file_system(&temp, false);

        let copy_op = CopyOperation {
            id: 12345,
            media_type: MediaType::Movie,
            title: "The Matrix".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };

        let result = file_system.inbox_folder(&copy_op);
        let expected = temp.path().join("inbox/0x00003039.The Matrix.D1");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_movie_inbox_folder_with_large_id() {
        let temp = TempFile::new("artie.test.fs.test_movie_inbox_folder_with_large_id");
        let file_system = setup_file_system(&temp, false);

        let copy_op = CopyOperation {
            id: 0xFFFFFFFF,
            media_type: MediaType::Movie,
            title: "Test Movie".to_string(),
            disc: 2,
            season: 0,
            ..CopyOperation::default()
        };

        let result = file_system.inbox_folder(&copy_op);
        let expected = temp.path().join("inbox/0xFFFFFFFF.Test Movie.D2");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_show_inbox_folder() {
        let temp = TempFile::new("artie.test.fs.test_show_inbox_folder");
        let file_system = setup_file_system(&temp, false);

        let copy_op = CopyOperation {
            id: 54321,
            media_type: MediaType::Show,
            title: "Breaking Bad".to_string(),
            disc: 3,
            season: 5,
            ..CopyOperation::default()
        };

        let result = file_system.inbox_folder(&copy_op);
        let expected = temp.path().join("inbox/0x0000D431.Breaking Bad.S5.D3");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_show_inbox_folder_season_zero() {
        let temp = TempFile::new("artie.test.fs.test_show_inbox_folder_season_zero");
        let file_system = setup_file_system(&temp, false);

        let copy_op = CopyOperation {
            id: 100,
            media_type: MediaType::Show,
            title: "Extras".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };

        let result = file_system.inbox_folder(&copy_op);
        let expected = temp.path().join("inbox/0x00000064.Extras.S0.D1");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_inbox_folder_with_special_characters_in_title() {
        let temp = TempFile::new(
            "artie.test.fs.test_inbox_folder_with_special_characters_in_title"
        );
        let file_system = setup_file_system(&temp, false);

        let copy_op = CopyOperation {
            id: 999,
            media_type: MediaType::Movie,
            title: "Movie: The Sequel!".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };

        let result = file_system.inbox_folder(&copy_op);
        let expected = temp.path().join("inbox/0x000003E7.Movie: The Sequel!.D1");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_inbox_folder_with_id_zero() {
        let temp = TempFile::new("artie.test.fs.test_inbox_folder_with_id_zero");
        let file_system = setup_file_system(&temp, false);

        let copy_op = CopyOperation {
            id: 0,
            media_type: MediaType::Movie,
            title: "Zero".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };

        let result = file_system.inbox_folder(&copy_op);
        let expected = temp.path().join("inbox/0x00000000.Zero.D1");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_inbox_path() {
        let temp = TempFile::new("artie.test.fs.test_inbox_path");
        let file_system = setup_file_system(&temp, false);

        let location = MediaLocation::Inbox(PathBuf::from("movie_folder"));

        let result = file_system.path(&location);

        assert_eq!(result, Some(temp.path().join("inbox/movie_folder")));
    }

    #[test]
    fn test_library_path() {
        let temp = TempFile::new("artie.test.fs.test_library_path");
        let file_system = setup_file_system(&temp, false);

        let location = MediaLocation::Library(PathBuf::from("show_folder"));

        let result = file_system.path(&location);

        assert_eq!(result, Some(temp.path().join("library/show_folder")));
    }

    #[test]
    fn test_archive_path() {
        let temp = TempFile::new("artie.test.fs.test_archive_path");
        let file_system = setup_file_system(&temp, false);

        let location = MediaLocation::Archive(PathBuf::from("archived_content"));

        let result = file_system.path(&location);

        assert_eq!(result, Some(temp.path().join("archive/archived_content")));
    }

    #[test]
    fn test_deleted_path() {
        let temp = TempFile::new("artie.test.fs.test_deleted_path");
        let file_system = setup_file_system(&temp, false);

        let location = MediaLocation::Deleted;

        let result = file_system.path(&location);

        assert_eq!(result, None);
    }

    #[test]
    fn test_inbox_path_with_nested_structure() {
        let temp = TempFile::new("artie.test.fs.test_inbox_path_with_nested_structure");
        let file_system = setup_file_system(&temp, false);

        let location = MediaLocation::Inbox(PathBuf::from("folder/subfolder/file"));

        let result = file_system.path(&location);

        assert_eq!(result, Some(temp.path().join("inbox/folder/subfolder/file")));
    }

    #[test]
    fn test_library_path_with_empty_path() {
        let temp = TempFile::new("artie.test.fs.test_library_path_with_empty_path");
        let file_system = setup_file_system(&temp, false);

        let location = MediaLocation::Library(PathBuf::from(""));

        let result = file_system.path(&location);

        assert_eq!(result, Some(temp.path().join("library/")));
    }
}

