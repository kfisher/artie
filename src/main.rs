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

// TODO: Need to determine the actual ID. Make sure to update the resources configuration as well
//       to match.
pub const APP_ID: &str = "org.example.Artie";

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;

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

pub mod task {
    //! Utilities for running asynchronous tasks.

    use std::sync::OnceLock;

    use tokio::runtime::Runtime;
    use tokio::task::JoinHandle;

    /// Spawns a new asynchronous task returning the join handle for it.
    ///
    /// This is essentially just a drop-in for the tokio::spawn method which can't be used because
    /// the runtime is manually setup instead of using `tokio::main` macro.
    pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        runtime().spawn(future)
    }

    /// Gets the tokio runtime.
    ///
    /// On the first call, the runtime will be initialized.
    fn runtime() -> &'static Runtime {
        static RUNTIME: OnceLock<Runtime> = OnceLock::new();
        RUNTIME.get_or_init(|| {
            Runtime::new().expect("Failed to init runtime")
        })
    }
}
