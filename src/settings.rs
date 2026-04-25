// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::path;

/// The application configuration settings.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Settings {
    /// File path settings.
    pub paths: path::Settings,

    // TODO: Dont forget test.
    // /// Network settings.
    // pub net: crate::net::Settings,
}

impl Settings {
    /// Loads settings stored in a TOML file at the provided path.
    ///
    /// # Errors
    ///
    /// [`crate::Error::StdIo`] if the file cannot be read.
    ///
    /// [`crate::Error::TomlDeserialize`] if the file's content cannot be deserialized.
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let settings: Settings = toml::from_str(&contents)?;
        tracing::info!(?path, "settings loaded");
        Ok(settings)
    }

    /// Saves the settings to the TOML file at the provided path.
    ///
    /// # Errors
    ///
    /// [`crate::Error::StdIo`] if the file cannot be written to.
    ///
    /// [`crate::Error::TomlSerialize`] if the settings cannot be serialized.
    pub fn save(&self, path: &Path) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        let mut file = fs::File::create(path)?;
        file.write_all(toml_string.as_bytes())?;
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
            paths: crate::path::Settings {
                inbox: PathBuf::from("/inbox"),
                library: PathBuf::from("/library"),
                archive: PathBuf::from("/archive"),
                data: PathBuf::from("/data"),
            },
//          net: crate::net::Settings {
//              workers: vec![
//                  String::from("127.0.0.1:0001"),
//                  String::from("127.0.0.1:0002"),
//              ],
//          }
        };

        settings.save(path.path()).unwrap();

        let loaded_settings = Settings::from_file(path.path()).unwrap();

        assert_eq!(settings.paths.inbox, loaded_settings.paths.inbox);
        assert_eq!(settings.paths.library, loaded_settings.paths.library);
        assert_eq!(settings.paths.archive, loaded_settings.paths.archive);
        assert_eq!(settings.paths.data, loaded_settings.paths.data);

//      assert_eq!(2, loaded_settings.net.workers.len());
//      assert_eq!(settings.net.workers[0], loaded_settings.net.workers[0]);
//      assert_eq!(settings.net.workers[1], loaded_settings.net.workers[1]);
    }
}
