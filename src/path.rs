// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Utilities for creating application file paths.
//!
//! Most of the utility functions for constructing paths follow the following patterns:
//!
//! `*_path` will return a normal path to a file or directory.
//!
//! `*_location` will return a media location which is a path relative to one of the media
//! locations (see [`MediaLocation`]). Unlike normal file paths, locations are not host specific
//! which allows paths to be shared between the control and worker nodes.
//!
//! # Initialization
//!
//! The path settings need initialized by calling [`init`]. This should happen very early in the
//! application startup. Additionally, the settings are not expected to change after being
//! initialized.
//!
//! # Data Directory
//!
//! The data directory is where the application data files are stored such as the SQLite database.
//! The following utility functions can be used to get file paths within this directory:
//!
//! - [`data_exists`] - Check if the data directory exists.
//! - [`data_path`] - Get the file path for a file within the data folder.
//!
//! # Inbox Directory
//!
//! The inbox directory is the application working directory. It is where the video files will be
//! created when copying and where the transcoded versions will be stored. Any data files created
//! during this process will also be stored here.
//!
//! - [`inbox_exists`] - Check if the inbox directory exists.
//! - [`inbox_location`] - Get the media location of a file in the inbox using a copy operation.
//! - [`inbox_path`] - Get the path of a file in the inbox using a copy operation.
//! - [`mkv_copy_log_location`] - Get the media location of the log created when copying a disc.
//! - [`mkv_info_log_location`] - Get the media location of the log created when getting disc info.
//! - [`disc_info_path`] - Get the path of the file containing disc info extracted from the disc.
//!
//! # Library
//!
//! The library directory is where the media server will search for media files. The video files
//! will be moved here when cataloged.
//!
//! - [`library_exists`] - Check if the library directory exists.
//!
//! # Archive
//!
//! The archive directory is where video and data files not needed by the media server, but are
//! kept for future reference. For example the user may wish to keep the MKV file that was created
//! when copied as a backup.
//!
//! - [`archive_exists`] - Check if the archive directory exists.
//!
//! # Panics
//!
//! Most of this API will panic if the module is not initialized. This is supposed to happen very
//! early in the application startup before anything else is initialized.

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::models::{CopyOperation, MediaLocation, MediaType};

/// Name of the file that is used to save the disc information extracted by MakeMKV.
pub const DISC_INFO_FILENAME: &str = "disc_info.json";

/// Name of the file that is used to log MakeMKV output when running the info command.
pub const MAKEMKV_INFO_LOG_FILENAME: &str = "makemkv-info.log";

/// Name of the file that is used to log MakeMKV output when running the copy (mkv) command.
pub const MAKEMKV_COPY_LOG_FILENAME: &str = "makemkv-copy.log";

/// The file path settings.
/// 
/// The settings are setup early in application initialization and are not expected to change
/// afterwards.
static PATH: OnceLock<Path> = OnceLock::new();

/// File path settings.
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

/// Returns `true` if the archive directory exists, is a directory, and accessible by the user
/// or `false` otherwise.
pub fn archive_exists() -> bool {
    PATH.get()
        .expect("path module not initialized")
        .archive_exists()
}

/// Returns `true` if the data directory exists, is a directory, and accessible by the user or
/// `false` otherwise.
pub fn data_exists() -> bool {
    PATH.get()
        .expect("path module not initialized")
        .data_exists()
}

/// Returns the path to a file with the provided name in the data directory.
pub fn data_path(name: &str) -> PathBuf {
    PATH.get()
        .expect("path module not initialized")
        .data_path(name)
}

/// Returns the path to the disc info file for a copy operation.
pub fn disc_info_path(copy_operation: &CopyOperation) -> PathBuf {
    PATH.get()
        .expect("path module not initialized")
        .disc_info_path(copy_operation)
}

/// Returns `true` if the inbox directory exists, is a directory, and accessible by the user or
/// `false` otherwise.
pub fn inbox_exists() -> bool {
    PATH.get()
        .expect("path module not initialized")
        .inbox_exists()
}

/// Returns the path to the inbox folder for a copy operation.
pub fn inbox_path(copy_operation: &CopyOperation) -> PathBuf {
    PATH.get()
        .expect("path module not initialized")
        .inbox_path(copy_operation)
}

/// Returns the media location for a file or folder in the inbox.
///
/// # Args
///
/// `copy_operation`:  The copy operation the file or folder was created for.
///
/// `filename`:  The name of the file or folder. If `None`, the location for the copy operation's
/// root inbox folder will be returned.
pub fn inbox_location(copy_operation: &CopyOperation, filename: Option<&str>) -> MediaLocation {
    let mut path = PathBuf::from(inbox_folder_name(copy_operation));
    if let Some(filename) = filename {
        path = path.join(filename);
    }
    MediaLocation::Inbox(path)
}

/// Initializes the path settings.
///
/// This must be called prior to any.
pub fn init(settings: Settings) -> Result<()> {
    let path = PATH.get_or_init(|| settings.into());
    path.make_dirs()
}

/// Returns `true` if the library directory exists, is a directory, and accessible by the user or
/// `false` otherwise.
pub fn library_exists() -> bool {
    PATH.get()
        .expect("path module not initialized")
        .library_exists()
}

/// Returns the path for a media location.
pub fn location_path(location: &MediaLocation) -> Option<PathBuf> {
    PATH.get()
        .expect("path module not initialized")
        .location_path(location)
}

/// Returns the location of the file to the log file created when gathering disc info during a copy
/// operation.
pub fn mkv_info_log_location(copy_operation: &CopyOperation) -> MediaLocation {
    inbox_location(copy_operation, Some(MAKEMKV_INFO_LOG_FILENAME))
}

/// Returns the location of the file to the log file created when copying a disc.
pub fn mkv_copy_log_location(copy_operation: &CopyOperation) -> MediaLocation {
    inbox_location(copy_operation, Some(MAKEMKV_COPY_LOG_FILENAME))
}

// NOTE: The Path struct was created to enable some level of testing given that the public API uses
//       a global variable. Any API function that relies on an application setting will essentially
//       just be a wrapper around the Path method. 

/// Utility for performing settings specific path operations.
struct Path {
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

impl Path {
    /// Returns `true` if the archive directory exists, is a directory, and accessible by the user
    /// or `false` otherwise.
    fn archive_exists(&self) -> bool {
        self.archive.is_dir()
    }

    /// Returns `true` if the data directory exists, is a directory, and accessible by the user or
    /// `false` otherwise.
    fn data_exists(&self) -> bool {
        self.data.is_dir()
    }

    /// Returns the path to a file with the provided name in the data directory.
    fn data_path(&self, name: &str) -> PathBuf {
        self.data.join(name)
    }

    /// Returns the path to the disc info file for a copy operation.
    fn disc_info_path(&self, copy_operation: &CopyOperation) -> PathBuf {
        self.inbox_path(copy_operation).join(DISC_INFO_FILENAME)
    }

    /// Returns `true` if the inbox directory exists, is a directory, and accessible by the user or
    /// `false` otherwise.
    fn inbox_exists(&self) -> bool {
        self.inbox.is_dir()
    }

    /// Returns the path to the inbox folder for a copy operation.
    fn inbox_path(&self, copy_operation: &CopyOperation) -> PathBuf {
        self.inbox.join(PathBuf::from(inbox_folder_name(copy_operation)))
    }

    /// Returns `true` if the library directory exists, is a directory, and accessible by the user or
    /// `false` otherwise.
    fn library_exists(&self) -> bool {
        self.library.is_dir()
    }

    /// Returns the path for a media location.
    fn location_path(&self, location: &MediaLocation) -> Option<PathBuf> {
        match location {
            MediaLocation::Inbox(path) => {
                Some(self.inbox.join(path))
            },
            MediaLocation::Library(path) => {
                Some(self.library.join(path))
            },
            MediaLocation::Archive(path) => {
                Some(self.archive.join(path))
            },
            MediaLocation::Deleted => None,
        }
    }

    /// Creates the folders if they don't already exist.
    ///
    /// # Errors
    ///
    /// [`crate::Error::StdIo`] will be returned if any of the create directory requests fails.
    fn make_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.inbox)?;
        fs::create_dir_all(&self.library)?;
        fs::create_dir_all(&self.archive)?;
        fs::create_dir_all(&self.data)?;
        Ok(())
    }
}

impl From<Settings> for Path {
    fn from(value: Settings) -> Self {
        Self {
            inbox: value.inbox,
            library: value.library,
            archive: value.archive,
            data: value.data,
        }
    }
}

/// Returns the name of the inbox folder based off the provided copy operation.
fn inbox_folder_name(copy_operation: &CopyOperation) -> String {
    match copy_operation.media_type {
        MediaType::Movie => {
            format!(
                "0x{:08X}.{}.D{}",
                copy_operation.id,
                copy_operation.title,
                copy_operation.disc
            )
        },
        MediaType::Show => {
            format!(
                "0x{:08X}.{}.S{}.D{}",
                copy_operation.id,
                copy_operation.title,
                copy_operation.season,
                copy_operation.disc
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::test_utils::TempDir;

    fn setup_test(temp: &TempDir, make_dirs: bool) -> Path {
        let base = temp.path();

        let settings = Settings {
            inbox: base.join("inbox"),
            library: base.join("library"),
            archive: base.join("archive"),
            data: base.join("data"),
        };

        let path: Path = settings.into();

        if make_dirs {
            path.make_dirs().unwrap();
        }

        path
    }

    #[test]
    fn test_archive_exists() {
        let temp = TempDir::new("artie.test.path.archive_exists");
        let path = setup_test(&temp, false);

        assert!(!path.archive_exists());
        fs::create_dir_all(&path.archive).unwrap();
        assert!(path.archive_exists());
    }

    #[test]
    fn test_data_exists() {
        let temp = TempDir::new("artie.test.path.data_exists");
        let path = setup_test(&temp, false);
  
        assert!(!path.data_exists());
        fs::create_dir_all(&path.data).unwrap();
        assert!(path.data_exists());
    }
  
    #[test]
    fn test_inbox_exists() {
        let temp = TempDir::new("artie.test.path.inbox_exists");
        let path = setup_test(&temp, false);
  
        assert!(!path.inbox_exists());
        fs::create_dir_all(&path.inbox).unwrap();
        assert!(path.inbox_exists());
    }
  
    #[test]
    fn test_library_exists() {
        let temp = TempDir::new("artie.test.path.library_exists");
        let path = setup_test(&temp, false);
  
        assert!(!path.library_exists());
        fs::create_dir_all(&path.library).unwrap();
        assert!(path.library_exists());
    }
  
    #[test]
    fn test_make_dirs_success() {
        let temp = TempDir::new("artie.test.path.make_dirs_success");
        let path = setup_test(&temp, false);
  
        // Create the directories
        let result = path.make_dirs();
        assert!(result.is_ok());
  
        // Verify all directories exist
        assert!(path.inbox.exists());
        assert!(path.library.exists());
        assert!(path.archive.exists());
        assert!(path.data.exists());
    }
  
    #[test]
    fn test_make_dirs_with_nested_paths() {
        let temp = TempDir::new("artie.test.path.make_dirs_with_nested_paths");
        let path = setup_test(&temp, false);
  
        let result = path.make_dirs();
        assert!(result.is_ok());
  
        // Verify all directories exist, including parent directories
        assert!(path.inbox.exists());
        assert!(path.library.exists());
        assert!(path.archive.exists());
        assert!(path.data.exists());
    }
  
    #[test]
    fn test_make_dirs_already_exist() {
        let temp = TempDir::new("artie.test.path.make_dirs_already_exists");
        let path = setup_test(&temp, false);
  
        // Create directories first time
        path.make_dirs().unwrap();
  
        // Create directories second time (should succeed without error)
        let result = path.make_dirs();
        assert!(result.is_ok());
  
        // Verify all directories still exist
        assert!(path.inbox.exists());
        assert!(path.library.exists());
        assert!(path.archive.exists());
        assert!(path.data.exists());
    }
  
    #[test]
    fn test_make_dirs_are_actually_directories() {
        let temp = TempDir::new("artie.test.path.make_dirs_actually_directories");
        let path = setup_test(&temp, false);
  
        path.make_dirs().unwrap();
  
        // Verify they are directories, not files
        assert!(path.inbox.is_dir());
        assert!(path.library.is_dir());
        assert!(path.archive.is_dir());
        assert!(path.data.is_dir());
    }
  
    #[test]
    fn test_make_dirs_with_file_conflict() {
        let temp = TempDir::new("artie.test.path.make_dirs_with_file_conflict");
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
  
        let path: Path = settings.into();
  
        // This should fail because inbox is a file, not a directory
        let result = path.make_dirs();
        assert!(result.is_err());
    }
  
    #[test]
    fn test_movie_inbox_folder() {
        let temp = TempDir::new("artie.test.path.test_movie_inbox_folder");
        let path = setup_test(&temp, false);
  
        let copy_op = CopyOperation {
            id: 12345,
            media_type: MediaType::Movie,
            title: "The Matrix".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };
  
        let result = path.inbox_path(&copy_op);
        let expected = temp.path().join("inbox/0x00003039.The Matrix.D1");
  
        assert_eq!(result, expected);
    }
  
    #[test]
    fn test_movie_inbox_folder_with_large_id() {
        let temp = TempDir::new("artie.test.path.test_movie_inbox_folder_with_large_id");
        let path = setup_test(&temp, false);
  
        let copy_op = CopyOperation {
            id: 0xFFFFFFFF,
            media_type: MediaType::Movie,
            title: "Test Movie".to_string(),
            disc: 2,
            season: 0,
            ..CopyOperation::default()
        };
  
        let result = path.inbox_path(&copy_op);
        let expected = temp.path().join("inbox/0xFFFFFFFF.Test Movie.D2");
  
        assert_eq!(result, expected);
    }
  
    #[test]
    fn test_show_inbox_folder() {
        let temp = TempDir::new("artie.test.path.test_show_inbox_folder");
        let path = setup_test(&temp, false);
  
        let copy_op = CopyOperation {
            id: 54321,
            media_type: MediaType::Show,
            title: "Breaking Bad".to_string(),
            disc: 3,
            season: 5,
            ..CopyOperation::default()
        };
  
        let result = path.inbox_path(&copy_op);
        let expected = temp.path().join("inbox/0x0000D431.Breaking Bad.S5.D3");
  
        assert_eq!(result, expected);
    }
  
    #[test]
    fn test_show_inbox_folder_season_zero() {
        let temp = TempDir::new("artie.test.path.test_show_inbox_folder_season_zero");
        let path = setup_test(&temp, false);
  
        let copy_op = CopyOperation {
            id: 100,
            media_type: MediaType::Show,
            title: "Extras".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };
  
        let result = path.inbox_path(&copy_op);
        let expected = temp.path().join("inbox/0x00000064.Extras.S0.D1");
  
        assert_eq!(result, expected);
    }
  
    #[test]
    fn test_inbox_folder_with_special_characters_in_title() {
        let temp = TempDir::new(
            "artie.test.path.test_inbox_folder_with_special_characters_in_title"
        );
        let path = setup_test(&temp, false);
  
        let copy_op = CopyOperation {
            id: 999,
            media_type: MediaType::Movie,
            title: "Movie: The Sequel!".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };
  
        let result = path.inbox_path(&copy_op);
        let expected = temp.path().join("inbox/0x000003E7.Movie: The Sequel!.D1");
  
        assert_eq!(result, expected);
    }
  
    #[test]
    fn test_inbox_folder_with_id_zero() {
        let temp = TempDir::new("artie.test.path.test_inbox_folder_with_id_zero");
        let path = setup_test(&temp, false);
  
        let copy_op = CopyOperation {
            id: 0,
            media_type: MediaType::Movie,
            title: "Zero".to_string(),
            disc: 1,
            season: 0,
            ..CopyOperation::default()
        };
  
        let result = path.inbox_path(&copy_op);
        let expected = temp.path().join("inbox/0x00000000.Zero.D1");
  
        assert_eq!(result, expected);
    }
  
    #[test]
    fn test_inbox_path() {
        let temp = TempDir::new("artie.test.path.test_inbox_path");
        let path = setup_test(&temp, false);
  
        let location = MediaLocation::Inbox(PathBuf::from("movie_folder"));
  
        let result = path.location_path(&location);
  
        assert_eq!(result, Some(temp.path().join("inbox/movie_folder")));
    }
  
    #[test]
    fn test_library_path() {
        let temp = TempDir::new("artie.test.path.test_library_path");
        let path = setup_test(&temp, false);
  
        let location = MediaLocation::Library(PathBuf::from("show_folder"));
  
        let result = path.location_path(&location);
  
        assert_eq!(result, Some(temp.path().join("library/show_folder")));
    }
  
    #[test]
    fn test_archive_path() {
        let temp = TempDir::new("artie.test.path.test_archive_path");
        let path = setup_test(&temp, false);
  
        let location = MediaLocation::Archive(PathBuf::from("archived_content"));
  
        let result = path.location_path(&location);
  
        assert_eq!(result, Some(temp.path().join("archive/archived_content")));
    }
  
    #[test]
    fn test_deleted_path() {
        let temp = TempDir::new("artie.test.path.test_deleted_path");
        let path = setup_test(&temp, false);
  
        let location = MediaLocation::Deleted;
  
        let result = path.location_path(&location);
  
        assert_eq!(result, None);
    }
  
    #[test]
    fn test_inbox_path_with_nested_structure() {
        let temp = TempDir::new("artie.test.path.test_inbox_path_with_nested_structure");
        let path = setup_test(&temp, false);
  
        let location = MediaLocation::Inbox(PathBuf::from("folder/subfolder/file"));
  
        let result = path.location_path(&location);
  
        assert_eq!(result, Some(temp.path().join("inbox/folder/subfolder/file")));
    }
  
    #[test]
    fn test_library_path_with_empty_path() {
        let temp = TempDir::new("artie.test.path.test_library_path_with_empty_path");
        let path = setup_test(&temp, false);
  
        let location = MediaLocation::Library(PathBuf::from(""));
  
        let result = path.location_path(&location);
  
        assert_eq!(result, Some(temp.path().join("library/")));
    }

    // TODO: missing tests
}
