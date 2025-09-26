// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::widget::rule::{Catalog, FillMode, Style};

use crate::theme::Theme;

pub use iced::widget::rule::Rule;

/// The style classes used for the horizontal and vertical line widgets.
#[derive(Default)]
pub enum RuleClass {
    #[default]
    Default,
}

impl Catalog for Theme {
    type Class<'a> = RuleClass;

    fn default<'a>() -> Self::Class<'a> {
        RuleClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        let palette = self.palette();

        let style = Style {
            color: palette.overlay_0.into(),
            fill_mode: FillMode::Full,
            radius: 0.0.into(),
            snap: true,
        };

        match class {
            RuleClass::Default => style,
        }
    }
}

