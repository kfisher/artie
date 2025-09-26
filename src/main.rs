// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod settings;
mod screen;
mod theme;
mod widget;

use std::path::{Path, PathBuf};

use iced::Fill;
use iced::theme::Style;
use iced::widget::{Column, Row, Space};

use crate::settings::{ScaleFactor, Settings};
use crate::screen::copy::CopyScreen;
use crate::screen::transcode::TranscodeScreen;
use crate::screen::settings::SettingsScreen;
use crate::theme::Theme;
use crate::widget::Element;
use crate::widget::button;
use crate::widget::container::{Container, ContainerClass};

fn main() -> iced::Result {
    // TODO: This treats all errors as the same. It should only use the default settings if the 
    //       file does not exist. Otherwise, it should error out and exit.
    let settings = settings::load().unwrap_or_else(|_| Settings::default());

    iced::application(move || Artie::new(settings.clone()), Artie::update, Artie::view)
        .title("Artie")
        .scale_factor(Artie::scale_factor)
        .theme(Artie::theme)
        .style(Artie::style)
        .run()
}

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

/// Specifies the application messages.
///
/// Application messages are essentially the interactions of the application. Whenever the user 
/// interacts with the application, the interaction will trigger an application update. See
/// [`Artie::update`] for more information. Note that interactions are not necessarily limited to 
/// user interactions.
#[derive(Clone, Debug)]
pub enum Message {
    SetScaleFactor(ScaleFactor),
    SetTheme(Theme),
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
pub struct Artie {
    /// The application settings.
    settings: Settings,

    /// The current application screen being displayed.
    screen: Screen,
}

impl Artie {
    /// Creates a new [`Artie`] instance.
    fn new(settings: Settings) -> Self {
        Self {
            settings,
            screen: Screen::Copy(CopyScreen::default()),
        }
    }

    /// Sets the scaling factor for the application.
    fn scale_factor(&self) -> f32 {
        self.settings.general.scale_factor.into()
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

    fn style(&self, theme: &Theme) -> Style {
        let palette = theme.palette();
        Style {
            background_color: palette.mantle.into(),
            text_color: palette.text.into(),
        }
    }

    /// Returns the theme of the application.
    fn theme(&self) -> Theme {
        self.settings.general.theme
    }

    /// Processes interactions to update the state of the application.
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ToggleTheme => {
                self.settings.general.toggle_theme();

                // TODO: Need to handle the errors.
                let _ = settings::save(&self.settings);
            },
            Message::ViewCopyScreen => self.show_copy_screen(),
            Message::ViewSettingsScreen => self.show_settings_screen(),
            Message::ViewTranscodeScreen => self.show_transcode_screen(),
            _ => return match &mut self.screen {
                Screen::Copy(_) => iced::Task::none(),
                Screen::Settings(screen) => screen.update(&mut self.settings, message),
                Screen::Transcode(_) => iced::Task::none(),
            }
        }

        iced::Task::none()
    }

    /// Uses the current application state to generate the view.
    fn view(&self) -> Element<'_> {
        let (copy_active, transcode_active, settings_active) = match self.screen {
            Screen::Copy(_) => (true, false, false),
            Screen::Settings(_) => (false, false, true),
            Screen::Transcode(_) => (false, true, false),
        };

        let sidebar = Column::with_capacity(5)
            .push(button::nav_button(
                    "fontawesome.v7.solid.compact-disc",
                    Message::ViewCopyScreen,
                    "Copy",
                    copy_active))
            .push(button::nav_button(
                    "fontawesome.v7.solid.film",
                    Message::ViewTranscodeScreen,
                    "Transcode",
                    transcode_active))
            .push(button::nav_button(
                    "fontawesome.v7.solid.gear",
                    Message::ViewSettingsScreen,
                    "Settings",
                    settings_active))
            .push(Space::with_height(Fill))
            .push(button::nav_button(
                    "fontawesome.v7.solid.circle-half-stroke",
                    Message::ToggleTheme,
                    "Toggle Theme",
                    false))
            .spacing(4)
            .padding([4, 2]);

        let sidebar = Container::new(sidebar)
            .class(ContainerClass::Background(|t| t.palette().crust))
            .height(Fill);

        let content = match &self.screen {
            Screen::Copy(copy_screen) => copy_screen.view(),
            Screen::Settings(settings_screen) => settings_screen.view(&self.settings),
            Screen::Transcode(transcode_screen) => transcode_screen.view(),
        };

        Row::with_capacity(2)
            .push(sidebar)
            .push(content)
            .into()
    }
}

