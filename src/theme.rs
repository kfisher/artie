// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter, Result};


use serde::{Deserialize, Serialize};

pub mod color;
pub mod palette;

use crate::theme::palette::{Palette, DARK_PALETTE};

/// Defines the available themes for the application.
#[derive(Clone, Copy, Debug, Deserialize, Default, PartialEq, Serialize)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    /// All available themes.
    pub const ALL: &'static [Self] = &[
        Self::Dark,
        Self::Light,
    ];

    /// Returns the base colors for the theme.
    pub fn palette(&self) -> &Palette {
        match self {
            Theme::Dark => &DARK_PALETTE,
            Theme::Light => &DARK_PALETTE,
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Dark => write!(f, "Dark"),
        }
    }
}

impl iced::theme::Base for Theme {
    fn base(&self) -> iced::theme::Style {
        iced::theme::Style {
            background_color: palette::colors::NORD00,
            text_color: palette::colors::NORD06,
        }
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        Some(iced::theme::Palette {
            background: palette::colors::NORD00,
            text: palette::colors::NORD06,
            primary: palette::colors::NORD08,
            success: palette::colors::NORD14,
            warning: palette::colors::NORD12,
            danger: palette::colors::NORD11,
        })
    }
}

