// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::widget::text_input::{Catalog, Status, Style};

use crate::theme::Theme;

/// The style classes used for the text widget.
#[derive(Default)]
pub enum TextInputClass {
    // The default input style.
    #[default]
    Default,
}

impl Catalog for Theme {
    type Class<'a> = TextInputClass;

    fn default<'a>() -> Self::Class<'a> {
        TextInputClass::default()
    }

    fn style(&self, _class: &Self::Class<'_>, _status: Status) -> Style {
        todo!()
        // let palette = self.palette();

        // let active = Style {
        //     background: Background::Color(self.palette().sky.into()),
        //     border: Border {
        //         radius: 2.0.into(),
        //         width: 1.0,
        //         color: palette.text.into(),
        //     },
        //     icon: palette.pink.into(),
        //     placeholder: palette.green.into(),
        //     value: palette.text.into(),
        //     selection: palette.mauve.into(),
        // };

        // match status {
        //     Status::Active => active,
        //     Status::Hovered => Style {
        //         border: Border {
        //             color: palette.pink.into(),
        //             ..active.border
        //         },
        //         ..active
        //     },
        //     Status::Focused { .. } => Style {
        //         border: Border {
        //             color: palette.pink.into(),
        //             ..active.border
        //         },
        //         ..active
        //     },
        //     Status::Disabled => Style {
        //         background: Background::Color(palette.pink.into()),
        //         value: active.placeholder,
        //         placeholder: palette.pink.into(),
        //         ..active
        //     },
        // }
    }
}


