// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::widget::text::{Catalog, Style};

use crate::theme::Theme;
use crate::widget::Element;

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

