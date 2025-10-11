// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

use iced::border::{Border, Radius};
use iced::widget::text_input::{Catalog, Status, Style};

use crate::theme::Theme;
use crate::theme::palette::{ColorSet, Palette};

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

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        let palette = self.palette();

        match status {
            Status::Active | Status::Hovered | Status::Disabled => Style {
                background: palette.surface_2.color.into(),
                border: border(&palette.surface_2, palette, class),
                icon: crate::theme::palette::colors::TODO,
                placeholder: palette.surface_2.text.scale_alpha(0.5),
                value: palette.text.color,
                selection: palette.selection.color,
            },
            Status::Focused { is_hovered: _ } => Style {
                background: palette.surface_3.color.into(),
                border: border(&palette.surface_3, palette, class),
                icon: crate::theme::palette::colors::TODO,
                placeholder: palette.surface_3.text.scale_alpha(0.5),
                value: palette.text.color,
                selection: palette.selection.color,
            },
        }
    }
}

/// Creates the border style for text input.
fn border(base: &ColorSet, palette: &Palette, class: &TextInputClass) -> Border {
    let color = match class {
        TextInputClass::Default => base.border,
        TextInputClass::Invalid => palette.danger.color,
    };

    Border {
        color,
        width: 1.0,
        radius: Radius {
            top_left: 2.0,
            top_right: 2.0,
            bottom_right: 2.0,
            bottom_left: 2.0,
        },
    }
}

