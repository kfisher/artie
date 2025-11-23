// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Horizontal/Vertical Rules

use iced::widget::rule::{Catalog, FillMode, Style};

use crate::ui::theme::Theme;

pub use iced::widget::rule::Rule;

/// The style classes used for the horizontal and vertical line widgets.
#[derive(Default)]
pub enum RuleClass {
    /// Color of rule will be based on the background palette color.
    #[default]
    Background,

    /// Color of rule will be based on the surface 1 palette color.
    Surface1,
}

impl Catalog for Theme {
    type Class<'a> = RuleClass;

    fn default<'a>() -> Self::Class<'a> {
        RuleClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        let palette = self.palette();

        let color = match class {
            RuleClass::Background => palette.background.border,
            RuleClass::Surface1 => palette.surface_1.border,
        };

        Style {
            color,
            fill_mode: FillMode::Full,
            radius: 0.0.into(),
            snap: false,
        }
    }
}

