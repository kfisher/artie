// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

mod error;
mod context;
mod settings;
mod screen;
mod theme;
mod widget;

use std::path::{Path, PathBuf};

use iced::advanced::graphics::futures::event;
use iced::{Length, Subscription, Task};
use iced::{self, Event};
use iced::keyboard::{self, Event as KeyboardEvent};
use iced::keyboard::key::{self, Key};
use iced::theme::Style;
use iced::widget::{Column, Row, Space};

use copy_srv::CopyService;
use tracing::instrument::WithSubscriber;

use crate::error::{Error, Result};
use crate::context::Context;
use crate::settings::{ScaleFactor, Settings};
use crate::screen::copy::CopyScreen;
use crate::screen::transcode::TranscodeScreen;
use crate::screen::settings::{SettingsScreenMessage, SettingsScreen};
use crate::theme::Theme;
use crate::widget::Element;
use crate::widget::button;
use crate::widget::container::{Container, ContainerClass};
use crate::widget::dialog;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(Artie::new, Artie::update, Artie::view)
        .title("Artie")
        .scale_factor(Artie::scale_factor)
        .subscription(Artie::subscription)
        .theme(Artie::theme)
        .style(Artie::style)
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
    /// Close the open dialog cancelling any pending actions.
    CloseDialog,

    /// Deletes a copy service configuration.
    DeleteCopyService {
        index: usize,
    },

    /// User interface event (e.g. keyboard, mouse, touch, etc.)
    Event(Event),

    /// Changes the application's scale factor.
    SetScaleFactor(ScaleFactor),

    /// Changes the application's theme.
    SetTheme(Theme),

    /// Message specific to the settings screen only.
    SettingsScreen(SettingsScreenMessage),

    /// Toggles the application's theme between light and dark modes.
    ToggleTheme,

    /// Updates an existing copy service's configuration.
    UpdateCopyService {
        index: usize,
        name: String,
        serial_number: String,
    },

    /// View the screen used to copy media.
    ViewCopyScreen,

    /// View the screen used to configuration the application.
    ViewSettingsScreen,

    /// View the screen used to transcode video titles.
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
    /// The application context.
    context: Context,

    /// The current application screen being displayed.
    screen: Screen,
}

impl Artie {
    /// Creates a new [`Artie`] instance.
    fn new() -> Self {
        let mut artie = Self {
            context: match Context::from_config() {
                Ok(context) => context,
                // TODO: Need to correctly handle this error. It should only use the default values
                //       if the file wasn't found. Otherwise, it should exit.
                Err(_) => Context::default(),
            },
            screen: Screen::Copy(CopyScreen::default()),
        };
        artie.show_settings_screen();
        artie
    }

    /// Closes the open dialog.
    fn close_dialog(&mut self) {
        match &mut self.screen {
            Screen::Copy(_) => (),
            Screen::Settings(screen) => screen.dialog_closed(),
            Screen::Transcode(_) => (),
        }
    }

    /// Updates and saves the copy service settings and notifies the settings screen if the screen
    /// is currently being displayed.
    fn copy_service_changed(&mut self) -> Result<()> {
        // Update the settings. To keep things easy, simply recreate the settings data.
        self.context.settings.update_copy_services(&self.context.copy_services);

        // Save the settings to the config file.
        self.context.save_settings()?;

        // If the settings screen changed, notify it that the configuration for one or more copy 
        // services has changed.
        if let Screen::Settings(screen) = &mut self.screen {
            screen.copy_service_updated();
        }

        Ok(())
    }

    /// Handles keyboard events.
    ///
    /// - `Escape` Close the modal dialog.
    /// - `Tab` / `Shift+Tab` Change focus to the next input or the previous input.
    fn key_event(&mut self, event: &KeyboardEvent) -> Task<Message> {
        match event {
            KeyboardEvent::KeyPressed { 
                key: Key::Named(key::Named::Tab),
                modifiers,
                ..
            } => {
                if modifiers.shift() {
                    iced::widget::focus_previous()
                } else {
                    iced::widget::focus_next()
                }
            },
            KeyboardEvent::KeyPressed { 
                key: Key::Named(key::Named::Escape),
                .. 
            } => {
                self.close_dialog();
                Task::none()
            },
            _ => Task::none(),
        }
    }

    /// Sets the scaling factor for the application.
    fn scale_factor(&self) -> f32 {
        self.context.settings.general.scale_factor.into()
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

    /// Subscribes to events.
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    /// Returns the base style of the application.
    fn style(&self, theme: &Theme) -> Style {
        let palette = theme.palette();
        Style {
            background_color: palette.background.color,
            text_color: palette.text.color,
        }
    }

    /// Returns the theme of the application.
    fn theme(&self) -> Theme {
        self.context.settings.general.theme
    }

    /// Processes interactions to update the state of the application.
    fn update(&mut self, message: Message) -> Task<Message> {
        // TODO: Consider adding logging output to each branch.
        let task: Result<Task<Message>> = match message {
            Message::CloseDialog => {
                self.close_dialog();
                Ok(Task::none())
            },
            Message::DeleteCopyService { index } => {
                self.close_dialog();
                self.context.copy_services.remove(index);
                self.copy_service_changed().map(|_| Task::none())
            },
            Message::Event(event) => match event {
                Event::Keyboard(event) => Ok(self.key_event(&event)),
                _ => Ok(Task::none())
            },
            Message::SetScaleFactor(factor) => {
                if self.context.settings.general.scale_factor != factor {
                    self.context.settings.general.scale_factor = factor;
                    self.context.save_settings().map(|_| Task::none())
                } else {
                    Ok(Task::none())
                }
            },
            Message::SetTheme(theme) => {
                if self.context.settings.general.theme != theme {
                    self.context.settings.general.theme = theme;
                    self.context.save_settings().map(|_| Task::none())
                } else {
                    Ok(Task::none())
                }
            },
            Message::SettingsScreen(message) => {
                if let Screen::Settings(screen) = &mut self.screen {
                    screen.process_message(&self.context, message);
                }
                Ok(Task::none())
            },
            Message::ToggleTheme => {
                self.context.settings.general.toggle_theme();
                self.context.save_settings().map(|_| Task::none())
            },
            Message::UpdateCopyService { index, name, serial_number } => {
                if index < self.context.copy_services.len() {
                    self.context.copy_services[index].update_config(&name, &serial_number)
                        .map_err(|error| Error::CopyServiceInitError { error })
                        .and_then(|_| self.copy_service_changed())
                        .map(|_| Task::none())
                } else {
                    let service = CopyService::new(&name, &serial_number);
                    match service {
                        Ok(service) => {
                            self.context.copy_services.push(service);
                            self.copy_service_changed().map(|_| Task::none())
                        },
                        Err(error) => {
                            Err(Error::CopyServiceInitError { error })
                        },
                    }
                }
            },
            Message::ViewCopyScreen => {
                self.show_copy_screen();
                Ok(Task::none())
            },
            Message::ViewSettingsScreen => {
                self.show_settings_screen();
                Ok(Task::none())
            },
            Message::ViewTranscodeScreen => {
                self.show_transcode_screen();
                Ok(Task::none())
            },
        };

        // TODO: Revisit the error handling. For now simply panic.
        task.expect("Update encountered and error while processing a message.")
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
            .push(Space::with_height(Length::Fill))

            .push(button::nav_button(
                "fontawesome.v7.solid.circle-half-stroke",
                Message::ToggleTheme,
                "Toggle Theme",
                false))
            .spacing(4)
            .padding([4, 2]);

        let sidebar = Container::new(sidebar)
            .class(ContainerClass::Background(|t| t.palette().surface_1.color))
            .height(Length::Fill);

        let (content, dialog) = match &self.screen {
            Screen::Copy(copy_screen) => (
                copy_screen.view(&self.context),
                None,
            ),
            Screen::Settings(settings_screen) => (
                settings_screen.view(&self.context),
                settings_screen.dialog(),
            ),
            Screen::Transcode(transcode_screen) => (
                transcode_screen.view(),
                None,
            ),
        };

        let content = Row::with_capacity(2)
            .push(sidebar)
            .push(content);

        if let Some(dialog) = dialog {
            dialog::view(content.into(), dialog)
        } else {
            content.into()
        }
    }
}

