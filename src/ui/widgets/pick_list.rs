// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Dropdown Selector Input Control

use iced::border::{Border, Radius};
use iced::widget::pick_list::{Catalog, Status, Style};

use crate::ui::theme::Theme;

pub use iced::widget::PickList;

/// The style classes used for the pick-list widget.
#[derive(Default)]
pub enum PickListClass {
    #[default]
    Default,
}

impl Catalog for Theme {
    type Class<'a> = PickListClass;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        PickListClass::default()
    }

    fn style(&self, _class: &<Self as Catalog>::Class<'_>, _status: Status) -> Style {
        let palette = self.palette();
        Style {
            text_color: palette.text.color,
            placeholder_color: palette.subtext.color,
            handle_color: palette.surface_2.text,
            background: palette.surface_2.color.into(),
            border: Border {
                color: palette.surface_2.border,
                width: 1.0,
                radius: Radius {
                    top_left: 2.0,
                    top_right: 2.0,
                    bottom_right: 2.0,
                    bottom_left: 2.0,
                },
            },
        }
    }
}
