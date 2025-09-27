// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::borrow::Cow;

use iced::border::width;
use iced::{Border, Length};
use iced::font::{Family, Font, Weight};
use iced::widget::{Column, Row, Space};
use iced::widget::container::Style as ContainerStyle;

use copy_srv::CopyService;

use optical_drive::{DiscState, OpticalDrive};

use crate::Element;
use crate::theme::Theme;
use crate::widget::container::{Container, ContainerClass};
use crate::widget::text::{self, Text};

/// Screen for copying titles from DVDs and Blu-rays.
pub struct CopyScreen {
}

impl CopyScreen {
    /// Create a new instance of the screen.
    pub fn new(_copy_services: &[CopyService]) -> CopyScreen {
        CopyScreen { }
    }

    /// Generates the view for the screen.
    pub fn view(&self, copy_services: &[CopyService]) -> Element<'_> {
        let mut drives: Vec<Element<'_>> = Vec::with_capacity(copy_services.len());
        for service in copy_services {
            let widget = DriveWidget {
                drive: service,
            };
            drives.push(widget.into());
        }

        Column::with_children(drives)
            .spacing(16)
            .padding([18, 36])
            .into()
    }
}

impl Default for CopyScreen {
    fn default() -> Self {
        Self::new(&Vec::new())
    }
}

/// Widget used to control copy operations for a drive.
struct DriveWidget<'a> {
    drive: &'a CopyService,
}

impl<'a> From<DriveWidget<'a>> for Element<'_> {
    fn from(widget: DriveWidget) -> Self {
        let header = Row::with_capacity(1)
            .push(drive_header_text(widget.drive.name.clone()))
            .width(Length::Fill);
        let header = Container::new(header)
            .padding([4, 12]);

        let footer = Row::with_capacity(3)
            .push(match &widget.drive.drive.disc {
                DiscState::None => drive_footer_text("No Disc"),
                DiscState::Inserted { label, uuid: _ } => drive_footer_text(label.clone()),
            })
            .push(Space::with_width(Length::Fill))
            .push(drive_footer_text(format!(
                "{} | {}",
                widget.drive.drive.path,
                widget.drive.drive.serial_number
            )))
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
