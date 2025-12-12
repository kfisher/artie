// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod context;
mod copy_srv;
mod db;
mod drive;
mod error;
mod fs;
mod models;
mod settings;
mod ui;

use gtk::{prelude::*, Button};
use gtk::{glib, Application, ApplicationWindow};

pub use crate::context::Context;
pub use crate::error::Error;

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;

const APP_ID: &str = "org.example.Artie";


fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(ui::build);

    // Run the application
    app.run()
}

