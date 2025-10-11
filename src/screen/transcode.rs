// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! [`crate::Screen::Transcode`] screen.

use crate::Element;
use crate::widget::text::Text;

/// Screen for transcoding titles from MKV to MP4.
pub struct TranscodeScreen {
}

impl TranscodeScreen {
    /// Create a new instance of the screen.
    pub fn new() -> TranscodeScreen {
        TranscodeScreen { }
    }

    /// Generates the view for the screen.
    pub fn view(&self) -> Element<'_> {
        Text::new("Transcode").into()
    }
}

impl Default for TranscodeScreen {
    fn default() -> Self {
        Self::new()
    }
}

