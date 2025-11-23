// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the user interface.

pub mod app;
pub mod screens;
pub mod theme;
pub mod widgets;

use std::time::Instant;

use iced::{self, Event};

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
    /// Cancels an active copy operation.
    CancelCopyDisc {
        index: usize,
    },

    /// Close the open dialog cancelling any pending actions.
    CloseDialog,

    /// Message to initiate a copy operation.
    CopyDisc {
        index: usize,
    },

    /// Message specific to the copy screen only.
    CopyScreen(CopyScreenMessage),

    /// Deletes a copy service configuration.
    DeleteCopyService {
        index: usize,
    },

    /// User interface event (e.g. keyboard, mouse, touch, etc.)
    Event(Event),

    /// Resets the copy service after a successful or failed copy operation.
    ResetCopyService {
        index: usize,
    },

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

