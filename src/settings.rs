// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Manages application settings.

use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::copy_srv::CopyService;
use crate::{Error, Result};
use crate::error::SerializationError;

/// Represents the display scaling factor of the application.
// TODO: Move to UI module?
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScaleFactor(f32);

impl ScaleFactor {
    /// All available scaling factors provided via the pick list.
    pub const OPTIONS: &'static [ScaleFactor] = &[
        ScaleFactor(1.0),
        ScaleFactor(1.5),
        ScaleFactor(2.0),
        ScaleFactor(2.5),
    ];
}

impl Display for ScaleFactor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        let ScaleFactor(ref value) = *self;
        write!(f, "{}%", value * 100.0)
    }
}

impl From<ScaleFactor> for f32 {
    fn from(value: ScaleFactor) -> Self {
        let ScaleFactor(scale_factor) = value;
        scale_factor
    }
}

/// Defines the general application settings.
#[derive(Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// The display scale factor the application.
    pub scale_factor: ScaleFactor,
}

impl GeneralSettings {
    /// Toggles the theme between light and dark themes.
    pub fn toggle_theme(&mut self) {
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            scale_factor: ScaleFactor::OPTIONS[0],
        }
    }
}

/// The application configuration settings.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Settings {
    /// General application settings.
    pub general: GeneralSettings,
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
    use std::env;
    use std::path::PathBuf;
    use std::thread;

    pub struct TempFile(pub PathBuf);

    impl TempFile {
        fn new(file_name: &Path) -> TempFile {
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
            let result = fs::remove_file(p);
            // Avoid panicking while panicking as this causes the process to immediately abort,
            // without displaying test results.
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }

    #[test]
    fn test_save_load_settings() {
        let path = TempFile::new(Path::new("artie.test.settings.toml"));

        let settings = Settings {
            general: GeneralSettings {
                scale_factor: ScaleFactor(1.5),
            },
        };

        settings.save(path.path()).unwrap();

        let loaded_settings = Settings::from_file(path.path()).unwrap();

        assert_eq!(settings.general.scale_factor.0, loaded_settings.general.scale_factor.0);
    }
}
