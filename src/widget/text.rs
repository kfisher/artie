// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Text

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

    // Inherits the text color from the parent.
    Inherit,

    // Subtext style.
    Subtext,
}

impl Catalog for Theme {
    type Class<'a> = TextClass;

    fn default<'a>() -> Self::Class<'a> {
        TextClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            TextClass::Default => Style {
                color: Some(self.palette().text.color),
            },
            TextClass::Inherit => Style {
                color: None,
            },
            TextClass::Subtext => Style {
                color: Some(self.palette().subtext.color),
            },
        }
    }
}

/// Creates level 2 heading text.
pub fn heading2<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .size(24)
        .font(Font {
            weight: Weight::Bold,
            ..Font::default()
        })
}

/// Creates label text.
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

/// Creates subtext.
pub fn small_subtext<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .class(TextClass::Subtext)
        .size(12)
}
