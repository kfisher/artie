// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use iced::Color;
use iced::border::{Border, Radius};
use iced::widget::container::{Catalog, Style};

use crate::theme::Theme;

pub use iced::widget::container::Container;

/// The style classes used for the container widget.
#[derive(Default)]
pub enum ContainerClass {
    /// Transparent container with no borders.
    #[default]
    Default,

    /// Container style for modal dialogues.
    Dialog,

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
        let palette = self.palette();
        match class {
            ContainerClass::Default => Style::default(),
            ContainerClass::Dialog => Style {
                background: Some(palette.modal.into()),
                ..Style::default()
            },
            ContainerClass::Background(f) => Style {
                background: Some(f(self).into()),
                ..Style::default()
            },
            ContainerClass::Panel => Style {
                background: Some(palette.surface_0.color.into()),
                border: Border {
                    color: palette.surface_0.border,
                    width: 1.0,
                    radius: Radius {
                        top_left: 4.0,
                        top_right: 4.0,
                        bottom_right: 4.0,
                        bottom_left: 4.0,
                    },
                },
                ..Style::default()
            },
            ContainerClass::Tooltip => Style {
                background: Some(palette.surface_1.color.scale_alpha(0.90).into()),
                border: Border {
                    color: palette.surface_1.border,
                    width: 1.0,
                    radius: Radius {
                        top_left: 4.0,
                        top_right: 4.0,
                        bottom_right: 4.0,
                        bottom_left: 4.0,
                    },
                },
                ..Style::default()
            },
            ContainerClass::Custom(f) => f(self),
        }
    }
}

