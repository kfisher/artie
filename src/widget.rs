// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

pub mod button;
pub mod container;
pub mod icon;
pub mod menu;
pub mod pick_list;
pub mod rule;
pub mod scrollable;
pub mod slider;
pub mod text;
pub mod text_input;

use crate::Message;
use crate::theme::Theme;

/// The base generic widget that all other widgets used in the application can be converted into.
pub type Element<'a> = iced::Element<'a, Message, Theme>;

