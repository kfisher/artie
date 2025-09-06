// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use iced::Element;
use iced::widget::text;

fn main() -> iced::Result {
    iced::application(Artie::default, Artie::update, Artie::view)
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
enum Message {
}

/// TODO: DOC
#[derive(Default)]
struct Artie {
}

impl Artie {
    /// Processes interactions to update the state of the application.
    fn update(&mut self, message: Message) {
        match message {
        }
    }

    /// Uses the current application state to generate the view.
    fn view(&self) -> Element<'_, Message> {
        text("Hello, Friend!").into()
    }
}


