// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod compress;
mod db;
mod drive;
mod error;
mod fs;
mod library;
mod models;
mod net;
mod settings;
mod ui;

#[cfg(test)]
pub(crate) mod test_utils;

use clap::{ArgAction, Parser};

use gtk::Application;
use gtk::gio;
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::glib;

use tracing::Level;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::*;

pub use crate::error::Error;

// TODO: Need to determine the actual ID. Make sure to update the resources configuration as well
//       to match.
pub const APP_ID: &str = "org.example.Artie";

/// Result type for the application.
pub type Result<T> = std::result::Result<T, Error>;

/// Specifies the application modes of operation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mode {
    /// Application is running as the control node instance.
    #[default]
    Control,

    /// Application is running as a worker node instance.
    Worker,

    // TODO: We'll eventually probably want a Headless mode which would be a worker without a GUI.
}

fn main() -> glib::ExitCode {

    // Command line arguments must be parsed before GTK is initialized since GTK may consume
    // std::env::args().
    let args = Args::parse();

    let filter = Targets::new()
        .with_target("artie", Level::DEBUG)
        .with_target("handbrake", Level::DEBUG)
        .with_target("makemkv", Level::DEBUG);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();

    let mode = if args.worker {
        let _ = crate::net::server::create_and_run_server(); // TODO: TEMP
        Mode::Worker
    } else {
        Mode::Control
    };

    tracing::info!(?mode, "starting");

    gio::resources_register_include!("compiled.gresource")
        .expect("Failed to register resources.");

    let app = Application::builder()
        // TODO: Comment this out for now for testing so that we can create multiple instances of
        //       the application to test networking.
        // TODO: Since single instance is the default, may need to revist some things to avoid
        //       extra instances of managers (e.g. client manager) if a second instance is started
        //       which seems to run some code on the single instance.
        //.application_id(APP_ID)
        .build();
    app.connect_activate(move |app| {
        ui::build(app, mode);
    });

    // Override the command line arguments. Otherwise, GTK will generate errors for the command
    // line arguments defined above. May need to revist to support GTK arguments.
    app.run_with_args(&["artie"])
}

#[derive(Parser, Debug)]
#[command(name = "artie", about = "Media library creation orchestration tool.")]
struct Args {
    /// Indicates that the application should be run as a worker node.
    #[arg(short = 'w', long = "worker", action = ArgAction::SetTrue)]
    worker: bool,
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

    /// Runs the provided closure on a thread where blocking is acceptable.
    ///
    /// This is essentially just a drop-in for the `tokio::spawn_blocking` method which can't be
    /// used because the runtime is manually setup instead of using `tokio::main` macro.
    pub fn spawn_blocking<F, R>(func: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        runtime().spawn_blocking(func)
    }

    /// Runs a future blocking until it completes.
    pub fn block_on<F>(future: F) -> F::Output
    where
        F: Future,
    {
        Runtime::new().expect("Failed to create blocking runtime").block_on(future)
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
