// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod copy_srv;
mod db;
mod drive;
mod error;
mod fs;
mod models;
mod settings;
mod ui;

use gtk::Application;
use gtk::gio;
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::glib;

pub use crate::error::Error;

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;

// TODO: Need to determine the actual ID. Make sure to update the resources configuration as well
//       to match.
const APP_ID: &str = "org.example.Artie";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();

    gio::resources_register_include!("compiled.gresource")
        .expect("Failed to register resources.");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();
    app.connect_activate(move |app| {
        ui::build(app);
    });
    app.run()
}

