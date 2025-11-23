// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Dialog Windows

use iced::Length;
use iced::widget::{Column, MouseArea, Row, Space};
use iced::widget::{self, Stack};

use crate::ui::{Element, Message};

use super::button::{Button, ButtonClass};
use super::container::{Container, ContainerClass};
use super::text::{self, Text};

/// Dialog for displaying a confirmation dialog.
pub struct ConfirmDeleteDialog<T> {
    pub id: T,
    pub text: String,
}

impl<T> ConfirmDeleteDialog<T> {
    /// Generate the UI element for displaying the dialog content.
    pub fn view(&self, message: Message) -> Element<'_> {
        let header = Container::new(text::heading2("Are you sure?"))
            .padding([16, 12]);

        let content = Container::new(
            Text::new(
                format!("This will permanently delete {} and cannot be undone.",
                self.text))
            )
            .class(ContainerClass::Background(|t| t.palette().surface_1.color))
            .width(Length::Fill)
            .padding([32, 12]);

        let confirm_button = Button::new(ButtonClass::Primary)
            .label("Yes")
            .width(100.0)
            .on_press(message);

        let cancel_button = Button::new(ButtonClass::Default)
            .label("No")
            .width(100.0)
            .on_press(Message::CloseDialog);

        let controls = Row::with_capacity(3)
            .push(Space::with_width(Length::Fill))
            .push(confirm_button)
            .push(cancel_button)
            .spacing(12)
            .padding([16, 12]);

        let content = Column::with_capacity(3)
            .push(header)
            .push(content)
            .push(controls)
            .padding([0, 1])
            .spacing(0);

        Container::new(content)
            .class(ContainerClass::Panel)
            .width(600)
            .into()
    }
}

/// Generates the UI element for displaying a modal dialog.
pub fn view<'a>(base: Element<'a>, content: Element<'a>) -> Element<'a>
{
    let dialog = Container::new(widget::opaque(content))
        .class(ContainerClass::Dialog)
        .center(Length::Fill);

    let dialog = widget::opaque(
        MouseArea::new(dialog).on_press(Message::CloseDialog)
    );

    Stack::new()
        .push(base)
        .push(dialog)
        .into()
}

