// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter, Result};

use crate::theme::Theme;

/// Represents the display scaling factor of the application.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
///
/// TODO: Need a way to load the settings from a file at startup and allow settings to be changed
///       by the user at runtime.
#[derive(Default)]
pub struct Settings {
    pub general: GeneralSettings,
}

