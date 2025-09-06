// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

mod views;

use iced::{Alignment, Element, Padding};
use iced::widget::{self, Column, Row};

fn main() -> iced::Result {
    iced::application(Artie::default, Artie::update, Artie::view)
        .scale_factor(Artie::scale_factor)
        .title("Artie")
        .run()
}

/// Specifies the application messages.
///
/// Application messages are essentially the interactions of the application. Whenever the user 
/// interacts with the application, the interaction will trigger an application update. See
/// [`Artie::update`] for more information. Note that interactions are not necessarily limited to
/// user interactions.
#[derive(Clone, Debug)]
pub enum Message {
    /// Change the application content to the Copy view.
    SelectCopyView,

    /// Change the application content to the Transcode view.
    SelectTranscodeView,
}

// TODO: DOC
enum View {
    Copy,
    Transcode,
}

/// Defines the application state.
struct Artie {
    view: View,
}

impl Default for Artie {
    fn default() -> Self {
        Artie {
            view: View::Copy,
        }
    }
}

impl Artie {
    /// Generates the application header row widget.
    fn header_row(&self) -> Row<'_, Message> {
        widget::row![
            widget::button("Copy")
                .on_press(Message::SelectCopyView),
            widget::button("Transcode")
                .on_press(Message::SelectTranscodeView),
        ]
            .align_y(Alignment::Center)
            .padding(Padding::from([4, 8]))
            .spacing(8)
    }

    // TODO: DOC
    fn scale_factor(&self) -> f32 {
        2.
    }

    /// Processes interactions to update the state of the application.
    fn update(&mut self, message: Message) {
        match message {
            Message::SelectCopyView => {
                self.view = View::Copy;
            },
            Message::SelectTranscodeView => {
                self.view = View::Transcode;
            },
        }
    }

    /// Uses the current application state to generate the view.
    fn view(&self) -> Element<'_, Message> {
        let header = self.header_row();

        let content = match self.view {
            View::Copy => views::copy::view(),
            View::Transcode => views::transcode::view(),
        };

        widget::column![
            header,
            widget::horizontal_rule(1),
            content,
        ]
        .into()
    }
}

