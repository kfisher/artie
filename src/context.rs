// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application context and related utilities.

use std::path::PathBuf;

use crate::{Error, Result};
use crate::settings::Settings;

// NOTE: Context is meant to store things that may need to be shared across various parts of the 
//       application. Its sort of a crude dependency injection system. In general, the items in 
//       here would apply to an application implemented without iced. The exception is the couple 
//       of settings that we want to persist.

/// Contains all of the application state data.
#[derive(Default)]
pub struct Context {
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
            //-] copy_services: Vec::new(),
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
    /// - [`Error::Serialization`] if the file's content cannot be deserialized.
    pub fn from_config() -> Result<Self> {
        let path = get_config_path();

        if !path.is_file() {
            return Err(Error::FileNotFound { path });
        }

        let settings = Settings::from_file(&path)?;

        let context = Self {
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
