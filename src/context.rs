// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application context and related utilities.

use std::path::PathBuf;
use std::rc::Rc;

use copy_srv::CopyService;

use db::Database;

use fs::FileSystem;

use crate::error::{Error, Result};
use crate::settings::Settings;

// NOTE: Context is meant to store things that may need to be shared across various parts of the 
//       application. Its sort of a crude dependency injection system. In general, the items in 
//       here would apply to an application implemented without iced. The exception is the couple 
//       of settings that we want to persist.

/// Contains all of the application state data.
#[derive(Default)]
pub struct Context {
    /// List of copy service instance.
    ///
    /// Each instance corresponds to an optical drive and is responsible for performing the copy 
    /// operations to copy titles from a disc.
    pub copy_services: Vec<CopyService>,

    /// Interface to the database.
    pub db: Rc<Database>,

    /// Provides utilities for interfacing with the file system.
    pub fs: FileSystem,

    /// The application settings.
    ///
    /// The application settings are saved to a TOML file. See [`get_config_path`] for more
    /// information on where this file is stored.
    pub settings: Settings,
}

impl Context {
    /// Creates a new [`Context`] instance with default values.
    pub fn new() -> Self {
        Self {
            copy_services: Vec::new(),
            db: Rc::new(Database::default()),
            fs: FileSystem::default(),
            settings: Settings::default(),
        }
    }

    /// Creates a new [`Context`] instance from the config.
    ///
    /// See [`get_config_path`] for more information on how the path is determined.
    ///
    /// # Errors
    ///
    /// - [`Error::FileIo`] if the file cannot be read, or
    /// - [`Error::FileNotFound`] if the file cannot be found, or
    /// - [`Error::InvalidArchivePath`] if the media archive folder does not exist or cannot be 
    ///   accessed, or
    /// - [`Error::InvalidDataPath`] if the data folder does not exist or cannot be accessed, or
    /// - [`Error::InvalidInboxPath`] if the media inbox folder does not exist or cannot be
    ///   accessed, or
    /// - [`Error::InvalidLibraryPath`] if the media library folder does not exist or cannot be
    ///   accessed, or
    /// - [`Error::Serialization`] if the file's content cannot be deserialized.
    pub fn from_config() -> Result<Self> {
        let path = get_config_path();

        if !path.is_file() {
            return Err(Error::FileNotFound { path });
        }

        let settings = Settings::from_file(&path)?;

        if !settings.fs.archive.is_dir() {
            return Err(Error::InvalidArchivePath { path: settings.fs.archive.clone() });
        }

        if !settings.fs.data.is_dir() {
            return Err(Error::InvalidDataPath { path: settings.fs.data.clone() });
        }

        if !settings.fs.inbox.is_dir() {
            return Err(Error::InvalidInboxPath { path: settings.fs.inbox.clone() });
        }

        if !settings.fs.library.is_dir() {
            return Err(Error::InvalidLibraryPath { path: settings.fs.library.clone() });
        }

        let fs = FileSystem::new(&settings.fs);

        let db = Rc::new(Database::new(settings.fs.data.as_ref()));

        let copy_services: Vec<CopyService> = settings.copy_services.iter()
            .map(|config| CopyService::new(&config.name, &config.serial_number, &db)
                // FIXME: Handle this error once the copy service can support having drives in a 
                //        disconnected state.
                .expect("Failed to create copy service!"))
            .collect();

        let context = Self {
            copy_services,
            db,
            fs,
            settings,
        };

        Ok(context)
    }

    /// Saves the settings to the config.
    ///
    /// This will create the file if it does not exist or overwrite the file if it does. See 
    /// [`get_config_path`] for more information on how the path is determined.
    ///
    /// # Errors
    ///
    /// - [`Error::FileIo`] if the file cannot be written to, or
    /// - [`Error::Serialization`] if the settings could not be serialized.
    pub fn save_settings(&self) -> Result<()> {
        let path = get_config_path();
        self.settings.save(&path)
    }
}

/// Get the path to the application's config file.
///
/// TODO: This currently just returns a hard-coded path for the purposes of development. It will
///       need to be updated to look at an environment variable first, then fallback to a standard
///       location based on the OS (e.g. ~/.config/artie or %AppData%/artie).
fn get_config_path() -> PathBuf {
    PathBuf::from("artie.toml")
}

// TODO: Tests:
// - from_config (working and error)
