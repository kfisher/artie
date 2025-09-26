// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::{Alignment, Length};
use iced::widget::{Column, Row};

use crate::{Element, Message};
use crate::settings::{ScaleFactor, Settings};
use crate::theme::Theme;
use crate::widget::container::{Container, ContainerClass};
use crate::widget::pick_list::PickList;
use crate::widget::rule::Rule;
use crate::widget::text;

/// Screen for managing the application settings.
pub struct SettingsScreen {
}

impl SettingsScreen {
    /// Create a new instance of the screen.
    pub fn new() -> Self {
        Self {
        }
    }

    /// Generates the view for the screen.
    pub fn view(&self, settings: &Settings) -> Element<'_> {
        let rows = Column::with_capacity(3)
            .push(text::heading1("General"))
            // .push(rule::horizontal())
            .push(self.general_settings_view(settings))
            .spacing(16);

        Container::new(rows)
            .class(ContainerClass::Default)
            .align_x(Alignment::Center)
            .padding([18, 36])
            .into()
    }

    /// Processes interactions related to the application settings.
    pub fn update(&mut self, settings: &mut Settings, message: Message) -> iced::Task<Message> {
        match message {
            Message::SetScaleFactor(factor) => settings.general.scale_factor = factor,
            Message::SetTheme(theme) => settings.general.theme = theme,
            _ => ()
        }

        iced::Task::none()
    }

    /// Generates the view for the general settings.
    fn general_settings_view(&self, settings: &Settings) -> Element<'_> {
        fn form_row<'a, T>(label: &'a str, control: T) -> Row<'a, Message, Theme> 
        where 
            T: Into<Element<'a>> + 'a
        {
            Row::with_capacity(2)
                .push(text::label(label).width(Length::Fill))
                .push(control)
                .align_y(Alignment::Center)
                .spacing(8)
                .padding(16)
                .width(Length::Fill)
        }

        let content = Column::with_capacity(3)
            .push(form_row(
                "Scale Factor",
                PickList::new(
                    ScaleFactor::OPTIONS,
                    Some(settings.general.scale_factor),
                    Message::SetScaleFactor
                ).width(100),
            ))
            .push(Rule::horizontal(1))
            .push(form_row(
                "Theme",
                PickList::new(
                    Theme::ALL,
                    Some(settings.general.theme),
                    Message::SetTheme
                ).width(100),
            ));

        Container::new(content)
            .class(ContainerClass::Panel)
            .max_width(1080)
            .into()
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}

