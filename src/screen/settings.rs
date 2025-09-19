// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Element, Message};
use crate::widget::text::Text;

/// Screen for managing the application settings.
pub struct SettingsScreen {
}

impl SettingsScreen {
    /// Create a new instance of the screen.
    pub fn new() -> SettingsScreen {
        SettingsScreen { }
    }

    /// Generates the view for the screen.
    pub fn view(&self) -> Element<'_> {
        Text::new("Settings").into()
    }
}


