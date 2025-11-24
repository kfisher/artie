// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the screen for configuring the application.

use iced::{Alignment, Length};
use iced::widget::{Column, Row, Space};

use tracing::error;

use crate::context::Context;
use crate::settings::ScaleFactor;
use crate::ui::{Element, Message};
use crate::ui::theme::Theme;
use crate::ui::widgets::button::{Button, ButtonClass};
use crate::ui::widgets::container::{Container, ContainerClass};
use crate::ui::widgets::dialog::ConfirmDeleteDialog;
use crate::ui::widgets::pick_list::PickList;
use crate::ui::widgets::rule::Rule;
use crate::ui::widgets::text;
use crate::ui::widgets::text_input::TextInput;

/// Messages specific to the settings screen.
#[derive(Clone, Debug)]
pub enum SettingsScreenMessage {
}

/// Screen for configuring application settings.
#[derive(Default)]
pub struct SettingsScreen {
}

impl SettingsScreen {
    /// Creates a new [`SettingsScreen`] instance.
    pub fn new() -> Self {
        Self { }
    }

    /// Generates a UI element for displaying a dialog.
    ///
    /// Will only return `Some` if the settings screen is displaying a dialog.
    pub fn dialog(&self) -> Option<Element<'_>> {
        None
    }

    /// Callback when a dialog is closed.
    ///
    /// This will close any dialog this screen may have been opened without applying any changes.
    pub fn dialog_closed(&mut self) {
    }

    /// Processes a settings screen message.
    pub fn process_message(&mut self, _ctx: &Context, message: SettingsScreenMessage) {
        match message {
        }
    }

    /// Generates the UI element for displaying the screen.
    pub fn view(&self, ctx: &Context) -> Element<'_> {
        let rows = Column::with_capacity(1)
            .push(self.appearance_view(ctx))
            .max_width(1080)
            .spacing(16);

        Container::new(rows)
            .class(ContainerClass::Default)
            .align_x(Alignment::Center)
            .padding([16, 32])
            .into()
    }

    /// Generates the UI element for displaying the section for the appearance settings.
    fn appearance_view(&self, ctx: &Context) -> Element<'_> {
        fn form_row<'a, T>(label: &'a str, control: T) -> Row<'a, Message, Theme> 
        where 
            T: Into<Element<'a>> + 'a
        {
            Row::with_capacity(2)
                .push(text::label(label).width(Length::Fill))
                .push(control)
                .align_y(Alignment::Center)
                .spacing(8)
                .width(Length::Fill)
                .padding([8, 0])
        }

        let header = Row::with_capacity(1)
            .push(text::heading2("Appearance"))
            .padding([8, 0]);

        let scale_factor = form_row(
            "Scale Factor",
            PickList::new(
                ScaleFactor::OPTIONS,
                Some(ctx.settings.general.scale_factor),
                Message::SetScaleFactor,
            )
            .width(100),
        );

        let theme = form_row(
            "Theme",
            PickList::new(
                Theme::ALL,
                Some(ctx.settings.general.theme),
                Message::SetTheme,
            )
            .width(100),
        );

        let content = Column::with_capacity(6)
            .push(header)
            .push(Rule::horizontal(1))
            .push(scale_factor)
            .push(Rule::horizontal(1))
            .push(theme)
            .push(Rule::horizontal(1));

        Container::new(content)
            .class(ContainerClass::Default)
            .into()
    }
}

