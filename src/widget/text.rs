// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;

use iced::font::{Font, Weight};
use iced::widget::text::{Catalog, Style};

use crate::theme::Theme;

pub use iced::widget::Text;

/// The style classes used for the text widget.
#[derive(Default)]
pub enum TextClass {
    // The default application text style.
    #[default]
    Default,
}

impl Catalog for Theme {
    type Class<'a> = TextClass;

    fn default<'a>() -> Self::Class<'a> {
        TextClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            TextClass::Default => Style {
                color: Some(self.palette().text.into()),
            },
        }
    }
}

/// Creates level 1 heading text.
pub fn heading1<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .size(32)
        .font(Font {
            weight: Weight::Bold,
            ..Font::default()
        })
}

/// Creates a label for a form.
pub fn label<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .size(16)
        .font(Font {
            weight: Weight::Bold,
            ..Font::default()
        })
}
