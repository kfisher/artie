// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};
use crate::theme::Theme;

/// Represents the display scaling factor of the application.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScaleFactor(f32);

impl ScaleFactor {
    /// All available scaling factors provided via the pick list.
    pub const OPTIONS: &'static [ScaleFactor] = &[
        ScaleFactor(1.0),
        ScaleFactor(2.0),
        ScaleFactor(3.0),
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

    /// The color theme (light or dark).
    pub theme: Theme,
}

impl GeneralSettings {
    /// Toggles the theme between light and dark themes.
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            scale_factor: ScaleFactor::OPTIONS[0],
            theme: Theme::Dark,
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
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let settings: Settings = toml::from_str(&contents)?;
        Ok(settings)
    }

    /// Saves the settings to the TOML file at the provided path.
    pub fn save(&self, path: &Path) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        let mut file = fs::File::create(path)?;
        file.write_all(toml_string.as_bytes())?;
        Ok(())
    }
}

/// Loads the settings from a TOML file.
pub fn load() -> Result<Settings> {
    // TODO: Don't hard-code. Should create an environment variable and then fallback to the
    //       standard OS config location.
    Settings::from_file(Path::new("artie.toml"))
}

/// Saves the settings to a TOML file.
pub fn save(settings: &Settings) -> Result<()> {
    // TODO: Don't hard-code. Should create an environment variable and then fallback to the
    //       standard OS config location.
    settings.save(Path::new("artie.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
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
                theme: Theme::Dark,
            },
        };

        settings.save(path.path()).unwrap();

        let loaded_settings = Settings::from_file(path.path()).unwrap();

        assert_eq!(settings.general.scale_factor.0, loaded_settings.general.scale_factor.0);
        assert_eq!(settings.general.theme, loaded_settings.general.theme);
    }
}
