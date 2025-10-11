// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

use iced::widget::slider::{Catalog, Status, Style};
use crate::theme::Theme;

/// The style classes used for the slider widget.
#[derive(Default)]
pub enum SliderClass {
    #[default]
    Default,
}

impl Catalog for Theme {
    type Class<'a> = SliderClass;

    fn default<'a>() -> Self::Class<'a> {
        SliderClass::default()
    }

    fn style(&self, _class: &Self::Class<'_>, _status: Status) -> Style {
        todo!()
        // let palette = self.palette();

        // let color_rb0 = palette.green;
        // let color_rb1 = palette.blue;
        // let color_hb0 = palette.pink;
        // let color_hb1 = palette.red;

        // Style {
        //     rail: Rail {
        //         backgrounds: (color_rb0.into(), color_rb1.into()),
        //         width: 4.0,
        //         border: Border::default(),
        //     },
        //     handle: Handle {
        //         shape: HandleShape::Circle { radius: 8.0 },
        //         background: color_hb0.into(),
        //         border_color: color_hb1.into(),
        //         border_width: 0.0,
        //     }
        // }
    }
}
