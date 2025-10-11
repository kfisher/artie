// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Progress Bars

use iced::border::{Border, Radius};
use iced::widget::progress_bar::{Catalog, Style};

use crate::theme::Theme;

pub use iced::widget::progress_bar::ProgressBar;

#[derive(Default)]
pub enum ProgressBarClass {
    #[default]
    Default,
}


impl Catalog for Theme {
    type Class<'a> = ProgressBarClass;

    fn default<'a>() -> Self::Class<'a> {
        ProgressBarClass::default()
    }

    fn style(&self, _class: &Self::Class<'_>) -> Style {
        let palette = self.palette();
        Style {
            background: palette.surface_2.color.into(),
            bar: palette.secondary.color.into(),
            border: Border {
                color: palette.surface_2.border,
                width: 1.0,
                radius: Radius {
                    top_left: 2.0,
                    top_right: 2.0,
                    bottom_right: 2.0,
                    bottom_left: 2.0,
                },
            },
        }
    }
}

