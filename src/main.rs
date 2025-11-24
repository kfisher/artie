// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod context;
mod db;
mod drive;
mod error;
mod fs;
mod models;
mod settings;
mod ui;

pub use crate::context::Context;
pub use crate::error::Error;

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    ui::app::run()
}

