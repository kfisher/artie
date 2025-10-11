// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

use iced::widget::scrollable::{Catalog, Rail, Scroller, Status, Style};
use iced::widget::container::Style as ContainerStyle;

use crate::theme::Theme;

/// The style classes used for the scrollable widget.
#[derive(Default)]
pub enum ScrollableClass {
    #[default]
    Default,
}

impl Catalog for Theme {
    type Class<'a> = ScrollableClass;

    fn default<'a>() -> Self::Class<'a> {
        ScrollableClass::default()
    }

    // TODO: Need to fill out the style. This was added as a requirement of another widget and
    //       currently doesn't seem to effect anything being used.
    fn style(&self, _class: &Self::Class<'_>, _status: Status) -> Style {
        let temporary: iced::Color = crate::theme::palette::colors::TODO;
        Style {
            container: ContainerStyle::default(),
            vertical_rail: Rail {
                background: None,
                border: iced::Border::default(),
                scroller: Scroller {
                    color: temporary,
                    border: iced::Border::default(),
                },
            },
            horizontal_rail: Rail {
                background: None,
                border: iced::Border::default(),
                scroller: Scroller {
                    color: temporary,
                    border: iced::Border::default(),
                },
            },
            gap: None,
        }
    }
}
