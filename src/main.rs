// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod screen;
mod theme;
mod widget;

use iced::Fill;
use iced::widget::{Column, Row, Space};

use crate::screen::{CopyScreen, TranscodeScreen, SettingsScreen};
use crate::theme::Theme;
use crate::widget::Element;
use crate::widget::button::{Button, ButtonClass};
use crate::widget::container::{Container, ContainerClass};

fn main() -> iced::Result {
    iced::application(Artie::default, Artie::update, Artie::view)
        .title("Artie")
        .scale_factor(Artie::scale_factor)
        .theme(Artie::theme)
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
    ToggleTheme,
    ViewCopyScreen,
    ViewSettingsScreen,
    ViewTranscodeScreen,
}

/// Specifies the main application screens that are toggled using the sidebar icons.
pub enum Screen {
    Copy(CopyScreen),
    Settings(SettingsScreen),
    Transcode(TranscodeScreen),
}


/// The application's state data.
struct Artie {
    /// General application settings.
    settings: Settings,

    /// The current application screen being displayed.
    screen: Screen,
}

impl Artie {
    /// Sets the scaling factor for the application.
    fn scale_factor(&self) -> f32 {
        self.settings.scale_factor
    }

    /// Change the application's main content to the Copy screen.
    fn show_copy_screen(&mut self) {
        self.screen = Screen::Copy(CopyScreen::new());
    }

    /// Change the application's main content to the Settings screen.
    fn show_settings_screen(&mut self) {
        self.screen = Screen::Settings(SettingsScreen::new());
    }

    /// Change the application's main content to the Transcode screen.
    fn show_transcode_screen(&mut self) {
        self.screen = Screen::Transcode(TranscodeScreen::new());
    }

    /// Returns the theme of the application.
    fn theme(&self) -> Theme {
        self.settings.theme
    }

    fn toggle_theme(&mut self) {
        self.settings.theme = match self.settings.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
    }

    /// Processes interactions to update the state of the application.
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ToggleTheme => self.toggle_theme(),
            Message::ViewCopyScreen => self.show_copy_screen(),
            Message::ViewSettingsScreen => self.show_settings_screen(),
            Message::ViewTranscodeScreen => self.show_transcode_screen(),
        }

        iced::Task::none()
    }

    /// Uses the current application state to generate the view.
    fn view(&self) -> Element<'_> {
        let sidebar = Column::with_capacity(5)
            .push(Button::new(ButtonClass::Nav)
                .icon("fontawesome.v7.solid.compact-disc")
                .on_press(Message::ViewCopyScreen)
                .tooltip("Copy".into()))
            .push(Button::new(ButtonClass::Nav)
                .icon("fontawesome.v7.solid.film")
                .on_press(Message::ViewTranscodeScreen)
                .tooltip("Transcode".into()))
            .push(Button::new(ButtonClass::Nav)
                .icon("fontawesome.v7.solid.gear")
                .on_press(Message::ViewSettingsScreen)
                .tooltip("Settings".into()))
            .push(Space::with_height(Fill))
            .push(Button::new(ButtonClass::Nav)
                .icon("fontawesome.v7.solid.circle-half-stroke")
                .on_press(Message::ToggleTheme)
                .tooltip("Toggle Theme".into()))
            .spacing(4)
            .padding([4, 2]);
        let sidebar = Container::new(sidebar)
            .class(ContainerClass::Background(|t| t.palette().crust))
            .height(Fill);

        let content = match &self.screen {
            Screen::Copy(copy_screen) => copy_screen.view(),
            Screen::Settings(settings_screen) => settings_screen.view(),
            Screen::Transcode(transcode_screen) => transcode_screen.view(),
        };

        Row::with_capacity(2)
            .push(sidebar)
            .push(content)
            .into()
    }
}

impl Default for Artie {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            screen: Screen::Copy(CopyScreen::new()),
        }
    }
}

/// Defines the general application settings.
///
/// TODO: Need a way to load the settings from a file at startup and allow settings to be changed
///       by the user at runtime.
struct Settings {
    /// The display scale factor the application.
    scale_factor: f32,

    /// The color theme (light or dark).
    theme: Theme,
}

impl Default for Settings {
    /// Returns the default application settings.
    fn default() -> Self {
        Settings {
            scale_factor: 1.5,
            theme: Theme::Dark,
        }
    }
}
