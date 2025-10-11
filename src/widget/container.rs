// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

use iced::Color;
use iced::border::{Border, Radius};
use iced::widget::container::{Catalog, Style};

use crate::theme::Theme;

pub use iced::widget::container::Container;

/// The style classes used for the container widget.
#[derive(Default)]
pub enum ContainerClass {
    // TODO: DOC
    Accent,

    /// Container with a background color.
    /// TODO: Deprecate and replace.
    Background(fn(&Theme) -> Color),

    /// Custom container style.
    Custom(fn(&Theme) -> Style),

    // TODO: DOC
    Danger,

    /// Transparent container with no borders.
    #[default]
    Default,

    /// Container style for modal dialogues.
    Dialog,

    /// Container style for panels.
    Panel,

    // TODO: DOC
    Secondary,

    // TODO: DOC
    Success,

    // TODO: DOC
    Surface0,

    // TODO: DOC
    Surface1,

    // TODO: DOC
    Surface2,

    // TODO: DOC
    Surface3,

    /// Container used as a tooltip.
    Tooltip,
}

impl Catalog for Theme {
    type Class<'a> = ContainerClass;

    fn default<'a>() -> Self::Class<'a> {
        ContainerClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        let palette = self.palette();
        match class {
            ContainerClass::Accent => Style {
                background: Some(palette.accent.color.into()),
                text_color: Some(palette.accent.text),
                ..Style::default()
            },
            ContainerClass::Custom(f) => f(self),
            ContainerClass::Danger => Style {
                background: Some(palette.danger.color.into()),
                text_color: Some(palette.danger.text),
                ..Style::default()
            },
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
            ContainerClass::Secondary => Style {
                background: Some(palette.secondary.color.into()),
                text_color: Some(palette.secondary.text),
                ..Style::default()
            },
            ContainerClass::Success => Style {
                background: Some(palette.success.color.into()),
                text_color: Some(palette.success.text),
                ..Style::default()
            },
            ContainerClass::Surface0 => Style {
                background: Some(palette.surface_0.color.into()),
                text_color: Some(palette.surface_0.text),
                ..Style::default()
            },
            ContainerClass::Surface1 => Style {
                background: Some(palette.surface_1.color.into()),
                text_color: Some(palette.surface_1.text),
                ..Style::default()
            },
            ContainerClass::Surface2 => Style {
                background: Some(palette.surface_2.color.into()),
                text_color: Some(palette.surface_2.text),
                ..Style::default()
            },
            ContainerClass::Surface3 => Style {
                background: Some(palette.surface_3.color.into()),
                text_color: Some(palette.surface_3.text),
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
        }
    }
}

