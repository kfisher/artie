// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use crate::Element;
use crate::widget::text::Text;

/// Screen for copying titles from DVDs and Blu-rays.
pub struct CopyScreen {
}

impl CopyScreen {
    /// Create a new instance of the screen.
    pub fn new() -> CopyScreen {
        CopyScreen { }
    }

    /// Generates the view for the screen.
    pub fn view(&self) -> Element<'_> {
        Text::new("Copy").into()
    }
}

impl Default for CopyScreen {
    fn default() -> Self {
        Self::new()
    }
}

