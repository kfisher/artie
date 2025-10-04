// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use iced::Border;
use iced::widget::text_input::{Catalog, Status, Style};

use crate::theme::Theme;

pub use iced::widget::text_input::TextInput;

/// The style classes used for the text widget.
#[derive(Default)]
pub enum TextInputClass {
    // The default input style.
    #[default]
    Default,

    Invalid,
}

impl Catalog for Theme {
    type Class<'a> = TextInputClass;

    fn default<'a>() -> Self::Class<'a> {
        TextInputClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, _status: Status) -> Style {
        let palette = self.palette();

        let border_color = match class {
            TextInputClass::Default => palette.border,
            TextInputClass::Invalid => palette.danger,
        };

        Style {
            background: iced::Background::Color(palette.surface_0.into()),
            border: self.border().color(border_color),
            icon: palette.green.into(),
            placeholder: palette.subtext_0.into(),
            value: palette.text.into(),
            selection: palette.green.into(),
        }
    }
}


