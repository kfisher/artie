// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Manages application settings.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};
use crate::error::SerializationError;

/// The application configuration settings.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Settings {
    /// File path settings.
    pub fs: crate::fs::Settings,
}

impl Settings {
    /// Loads settings stored in a TOML file at the provided path.
    ///
    /// # Errors
    ///
    /// - [`Error::FileIo`] if the file cannot be read, or
    /// - [`Error::Serialization`] if the file's content cannot be deserialized.
    pub fn from_file(path: &Path) -> Result<Self> {

        let contents = fs::read_to_string(path)
            .map_err(|error| Error::FileIo { path: path.to_owned(), error })?;

        let settings: Settings = toml::from_str(&contents)
            .map_err(|error| Error::Serialization { 
                path: Some(path.to_owned()),
                error: SerializationError::TomlDeserialize(error),
            })?;

        tracing::info!(?path, "settings loaded");
        Ok(settings)
    }

    /// Saves the settings to the TOML file at the provided path.
    ///
    /// # Errors
    ///
    /// - [`Error::FileIo`] if the file cannot be written to, or
    /// - [`Error::Serialization`] if the settings cannot be serialized.
    pub fn save(&self, path: &Path) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|error| Error::Serialization { 
                path: Some(path.to_owned()),
                error: SerializationError::TomlSerialize(error),
            })?;
        let mut file = fs::File::create(path)
            .map_err(|error| Error::FileIo { path: path.to_owned(), error })?;
        file.write_all(toml_string.as_bytes())
            .map_err(|error| Error::FileIo { path: path.to_owned(), error })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::test_utils::TempFile;

    #[test]
    fn test_save_load_settings() {
        let path = TempFile::new(Path::new("artie.test.settings.toml"));

        let settings = Settings {
            fs: crate::fs::Settings {
                inbox: PathBuf::from("/inbox"),
                library: PathBuf::from("/library"),
                archive: PathBuf::from("/archive"),
                data: PathBuf::from("/data"),
            }
        };

        settings.save(path.path()).unwrap();

        let loaded_settings = Settings::from_file(path.path()).unwrap();

        assert_eq!(settings.fs.inbox, loaded_settings.fs.inbox);
        assert_eq!(settings.fs.library, loaded_settings.fs.library);
        assert_eq!(settings.fs.archive, loaded_settings.fs.archive);
        assert_eq!(settings.fs.data, loaded_settings.fs.data);
    }
}
