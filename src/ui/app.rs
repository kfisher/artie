// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! The main application window.

use std::time::{Duration, Instant};

use iced::advanced::graphics::futures::event;
use iced::{Length, Subscription, Task};
use iced::{self, Event};
use iced::keyboard::Event as KeyboardEvent;
use iced::keyboard::key::{self, Key};
use iced::theme::Style;
use iced::time;
use iced::widget::{Column, Row, Space};

use crate::{Context, Error, Result};

use super::{Element, Message};
use super::theme::Theme;
use super::screens::copy::CopyScreen;
use super::screens::transcode::TranscodeScreen;
use super::screens::settings::SettingsScreen;
use super::widgets::button;
use super::widgets::container::{Container, ContainerClass};
use super::widgets::dialog;
use super::workers;

/// Runs the application.
pub fn run() -> iced::Result {
    iced::application(Artie::new, Artie::update, Artie::view)
        .title("Artie")
        .scale_factor(Artie::scale_factor)
        .subscription(Artie::subscription)
        .theme(Artie::theme)
        .style(Artie::style)
        .run()
}

/// Specifies the main application screens that are toggled using the sidebar icons.
enum Screen {
    Copy(CopyScreen),
    Settings(SettingsScreen),
    Transcode(TranscodeScreen),
}

/// The application's state data.
struct Artie {
    /// The application context.
    context: Context,

    /// The current application screen being displayed.
    screen: Screen,

    /// The time of the last received tick.
    last_tick: Instant,

    /// Indicates if the tick event should be enabled.
    tick_enabled: bool,
}

impl Artie {
    /// Creates a new [`Artie`] instance.
    fn new() -> Self {
        let mut artie = Self {
            // TODO: Need to correctly handle this error. It should only use the default values
            //       if the file wasn't found. Otherwise, it should exit.
            context: Context::from_config().unwrap_or_default(),
            screen: Screen::Copy(CopyScreen::default()),
            last_tick: Instant::now(),
            tick_enabled: false,
        };
        artie.show_copy_screen();
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
        self.screen = Screen::Copy(CopyScreen::new(&self.context));
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
        let mut subscriptions = Vec::with_capacity(3);

        // Runtime events. Used for responding to key presses for the purposes of keyboard
        // shortcuts.
        subscriptions.push(event::listen().map(Message::Event));

        let subscription = Subscription::run(workers::drive_director)
            .map(Message::WorkerEvent);
        subscriptions.push(subscription);

        // Tick the UI if enabled. Will generally only be enabled when there is at least one 
        // active animation.
        if self.tick_enabled {
            subscriptions.push(
                time::every(Duration::from_secs_f32(1.0 / 60.0)).map(Message::Tick)
            );
        }

        Subscription::batch(subscriptions)
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
            Message::CopyScreen(message) => {
                if let Screen::Copy(screen) = &mut self.screen {
                    screen.process_message(&self.context, message);
                }
                Ok(Task::none())
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
            Message::Tick(instant) => {
                let delta_time = instant.duration_since(self.last_tick).as_secs_f32();
                self.last_tick = instant;
                if let Screen::Copy(screen) = &mut self.screen {
                    screen.tick(delta_time);
                }
                Ok(Task::none())
            },
            Message::ToggleTheme => {
                self.context.settings.general.toggle_theme();
                self.context.save_settings().map(|_| Task::none())
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
            _ => Ok(Task::none()) // FIXME
        };

        self.tick_enabled = match &self.screen {
            Screen::Copy(copy_screen) => copy_screen.should_tick(),
            _ => false,
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
            .push(content)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(dialog) = dialog {
            dialog::view(content.into(), dialog)
        } else {
            content.into()
        }
    }
}

