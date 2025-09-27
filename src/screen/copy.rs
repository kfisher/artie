// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::borrow::Cow;

use iced::border::width;
use iced::{Border, Length};
use iced::font::{Family, Font, Weight};
use iced::widget::{Column, Row, Space};
use iced::widget::container::Style as ContainerStyle;

use crate::Element;
use crate::theme::Theme;
use crate::widget::container::{Container, ContainerClass};
use crate::widget::text::{self, Text};

/// Screen for copying titles from DVDs and Blu-rays.
pub struct CopyScreen {
}

impl CopyScreen {
    /// Create a new instance of the screen.
    pub fn new() -> CopyScreen {
        CopyScreen { }
    }

    /// Generates the view for the screen.
    pub fn view(&self) -> Element<'_> {
        Column::with_capacity(3)
            .push(DriveWidget{})
            .push(DriveWidget{})
            .spacing(16)
            .padding([18, 36])
            .into()
    }
}

impl Default for CopyScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget used to control copy operations for a drive.
struct DriveWidget;

impl From<DriveWidget> for Element<'_> {
    fn from(_widget: DriveWidget) -> Self {
        let header = Row::with_capacity(1)
            .push(drive_header_text("DRIVE A"))
            .width(Length::Fill);
        let header = Container::new(header)
            .padding([4, 12]);

        let footer = Row::with_capacity(3)
            .push(drive_footer_text("MY_MOVIE"))
            .push(Space::with_width(Length::Fill))
            .push(drive_footer_text("example | /dev/sr0 | SN000001"))
            .width(Length::Fill);
        let footer = Container::new(footer)
            .padding([4, 12])
            .width(Length::Fill);

        let content = Container::new(Text::new("<<CONTENT>>"))
            .class(ContainerClass::Background(|t| t.palette().base))
            .padding(16)
            .width(Length::Fill);

        let content = Column::with_capacity(3)
            .push(header)
            .push(content)
            .push(footer)
            .padding([2, 1]);

        Container::new(content)
            .class(ContainerClass::Custom(|t| ContainerStyle {
                background: Some(t.palette().surface_0.into()),
                border: Border::default()
                    .width(1)
                    .color(t.palette().border)
                    .rounded(8),
                ..ContainerStyle::default()
            }))
            .max_width(1080)
            .into()
    }
}

/// Creates a text widget for the drive widget header.
pub fn drive_header_text<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .size(16)
        .font(Font {
            weight: Weight::Bold,
            ..Font::default()
        })
}

/// Creates a text widget for the drive widget footer.
pub fn drive_footer_text<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .size(16)
        .font(Font {
            weight: Weight::Bold,
            family: Family::Monospace,
            ..Font::default()
        })
}
