// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Dropdown menu overlays.

use iced::border::{Border, Radius};
use iced::overlay::menu::{Catalog, Style};

use crate::ui::theme::Theme;

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
            background: palette.surface_2.color.into(),
            border: Border {
                color: palette.surface_2.border,
                width: 1.0,
                radius: Radius {
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_right: 0.0,
                    bottom_left: 0.0,
                },
            },
            text_color: palette.text.color,
            selected_text_color: palette.selection.text,
            selected_background: palette.selection.color.into(),
        }
    }
}

