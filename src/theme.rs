// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

pub mod color;
pub mod palette;

use crate::theme::color::Color;
use crate::theme::palette::{Palette, mocha as default_palette};

/// Defines the available themes for the application.
#[derive(Clone, Copy, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    /// Returns the base colors for the theme.
    pub const fn palette(&self) -> &Palette {
        match self {
            Theme::Dark => &Palette::DARK,
            Theme::Light => &Palette::LIGHT,
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

