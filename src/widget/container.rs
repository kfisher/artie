// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::border::Border;
use iced::widget::container::{Catalog, Style};

use crate::Message;
use crate::theme::Theme;
use crate::theme::color::Color;
use crate::widget::Element;

pub use iced::widget::container::Container;

/// The style classes used for the container widget.
#[derive(Default)]
pub enum ContainerClass {
    /// Transparent container with no borders.
    #[default]
    Default,

    /// Container with a background color.
    Background(fn(&Theme) -> Color),

    /// Container used as a tooltip.
    Tooltip,
}

impl Catalog for Theme {
    type Class<'a> = ContainerClass;

    fn default<'a>() -> Self::Class<'a> {
        ContainerClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        let style = Style::default();
        match class {
            ContainerClass::Default => style,
            ContainerClass::Background(f) => Style {
                background: Some(f(self).into()),
                ..style
            },
            ContainerClass::Tooltip => Style {
                background: Some(self.palette().crust.into()),
                border: Border::default()
                    .rounded(2)
                    .width(1)
                    .color(self.palette().primary),
                ..style
            }
        }
    }
}

