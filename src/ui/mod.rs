// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the user interface.

pub mod app;
pub mod screens;
pub mod theme;
pub mod widgets;
pub mod workers;

use std::time::Instant;

use crate::settings::ScaleFactor;

use screens::copy::CopyScreenMessage;
use screens::settings::SettingsScreenMessage;
use theme::Theme;

/// The base generic widget that all other widgets used in the application can be converted into.
pub type Element<'a> = iced::Element<'a, Message, Theme>;

/// Specifies the application messages.
///
/// Application messages are essentially the interactions of the application. Whenever the user 
/// interacts with the application, the interaction will trigger an application update.
#[derive(Clone, Debug)]
pub enum Message {
    /// Close the open dialog cancelling any pending actions.
    CloseDialog,

    /// Message specific to the copy screen only.
    CopyScreen(CopyScreenMessage),

    /// User interface event (e.g. keyboard, mouse, touch, etc.)
    Event(iced::Event),

    /// Changes the application's scale factor.
    SetScaleFactor(ScaleFactor),

    /// Changes the application's theme.
    SetTheme(Theme),

    /// Message specific to the settings screen only.
    SettingsScreen(SettingsScreenMessage),

    /// Emitted at a regular interval when tick is enabled.
    Tick(Instant),

    /// Toggles the application's theme between light and dark modes.
    ToggleTheme,

    /// View the screen used to copy media.
    ViewCopyScreen,

    /// View the screen used to configuration the application.
    ViewSettingsScreen,

    /// View the screen used to transcode video titles.
    ViewTranscodeScreen,

    /// Event from one of the worker routines.
    WorkerEvent(workers::Event),
}

