// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter, Result};

use iced::border::{Border, Radius};

use serde::{Deserialize, Serialize};

pub mod color;
pub mod palette;

use crate::theme::palette::{Palette, mocha as default_palette};

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
    pub const fn palette(&self) -> &Palette {
        match self {
            Theme::Dark => &Palette::DARK,
            Theme::Light => &Palette::LIGHT,
        }
    }

    /// Returns the standard border style.
    pub fn border(&self) -> Border {
        Border {
            color: self.palette().border.into(),
            width: 2.0,
            radius: Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
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
            background_color: default_palette::BASE.into(),
            text_color: default_palette::TEXT.into(),
        }
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        Some(iced::theme::Palette {
            background: default_palette::BASE.into(),
            text: default_palette::TEXT.into(),
            primary: default_palette::BLUE.into(),
            success: default_palette::GREEN.into(),
            warning: default_palette::YELLOW.into(),
            danger: default_palette::RED.into()
        })
    }
}

