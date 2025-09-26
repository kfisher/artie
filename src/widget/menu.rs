// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::Background;
use iced::border::Border;
use iced::overlay::menu::{Catalog, Style};

use crate::theme::Theme;

/// The style classes used for the menu widget.
#[derive(Default)]
pub enum MenuClass {
    #[default]
    Default,
}

impl Catalog for Theme {
    type Class<'a> = MenuClass;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        MenuClass::default()
    }

    fn style(&self, _class: &<Self as Catalog>::Class<'_>) -> Style {
        let palette = self.palette();
        Style {
            background: Background::Color(palette.overlay_0.into()),
            border: Border::default(),
            text_color: palette.text.into(),
            selected_text_color: palette.text.into(),
            selected_background: Background::Color(palette.primary.alpha(0.30).into()),
        }
    }
}
