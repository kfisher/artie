// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::widget::pick_list::{Catalog, Status, Style};

use crate::theme::Theme;

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
            text_color: palette.text.into(),
            placeholder_color: palette.sky.into(),
            handle_color: palette.text.into(),
            background: palette.surface_0.into(),
            border: self.border(),
        }
    }
}
