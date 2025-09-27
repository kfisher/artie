// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use iced::border::Border;
use iced::widget::container::{Catalog, Style};

use crate::theme::Theme;
use crate::theme::color::Color;

pub use iced::widget::container::Container;

/// The style classes used for the container widget.
#[derive(Default)]
pub enum ContainerClass {
    /// Transparent container with no borders.
    #[default]
    Default,

    /// Container with a background color.
    Background(fn(&Theme) -> Color),

    /// Container style for panels.
    Panel,

    /// Container used as a tooltip.
    Tooltip,

    /// Custom container style.
    Custom(fn(&Theme) -> Style),
}

impl Catalog for Theme {
    type Class<'a> = ContainerClass;

    fn default<'a>() -> Self::Class<'a> {
        ContainerClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            ContainerClass::Default => Style::default(),
            ContainerClass::Background(f) => Style {
                background: Some(f(self).into()),
                ..Style::default()
            },
            ContainerClass::Panel => Style {
                background: Some(self.palette().base.into()),
                border: Border::default()
                    .width(1)
                    .color(self.palette().border)
                    .rounded(8),
                ..Style::default()
            },
            ContainerClass::Tooltip => Style {
                background: Some(self.palette().crust.into()),
                border: Border::default()
                    .rounded(2)
                    .width(1)
                    .color(self.palette().primary),
                ..Style::default()
            },
            ContainerClass::Custom(f) => f(self),
        }
    }
}

