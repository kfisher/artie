// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod actor;
mod app;
mod compress;
mod error;
mod bus;
mod db;
mod drive;
mod library;
mod net;
mod path;
mod models;
mod settings;
mod task;
mod ui;

#[cfg(test)]
mod test_utils;

use std::path::PathBuf;

use clap::{ArgAction, Parser};

use tracing::Level;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::*;

pub use error::Error;

use net::client;
use net::server;
use settings::Settings;

/// Specifies the application's modes of operation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mode {
    /// Application is running as the control node instance.
    #[default]
    Control,

    /// Application is running as a worker node instance.
    Worker,
}

/// Result type for the application.
pub type Result<T> = core::result::Result<T, Error>;

/// Defines the command line arguments.
#[derive(Parser, Debug)]
#[command(name = "artie", about = "Media library creation orchestration tool.")]
struct Args {
    /// Indicates that the application should be run as a worker node.
    #[arg(short = 'w', long = "worker", action = ArgAction::SetTrue)]
    worker: bool,
}

/// Get the path to the application's config file.
///
/// TODO: This currently just returns a hard-coded path for the purposes of development. It will
///       need to be updated to look at an environment variable first, then fallback to a standard
///       location based on the OS (e.g. ~/.config/artie or %AppData%/artie).
fn get_config_path() -> PathBuf {
    PathBuf::from("artie.toml")
}

fn main() -> Result<()> {
    let args = Args::parse();


    let filter = Targets::new()
        .with_target("artie", Level::DEBUG)
        .with_target("handbrake", Level::DEBUG)
        .with_target("makemkv", Level::DEBUG);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();

    let mode = if args.worker {
        Mode::Worker
    } else {
        Mode::Control
    };

    tracing::info!(?mode, "starting");

    let settings = Settings::from_file(&get_config_path())?;

    path::init(settings.paths)?;

    // Initialize the message bus channel first. Bus initialization is done in two parts so that
    // the channel can be provided to the other actors.
    let (bus, bus_recv) = bus::init_channel();

    let db = db::init()?;
    let drive_mgr = drive::init(&bus)?;

    let net = if mode == Mode::Control {
        client::manager::init(&settings.net)
    } else {
        server::init()
    };

    // Start the message bus processing task.
    let join_handle = bus::init_processor(db, drive_mgr, net, bus_recv);

    // TODO: Eventually, we will want to use feature flags so that we can compile a version without
    //       the UI all together.

    if mode == Mode::Control {
        let _ = ui::run(mode, &bus)?;
    } else {
        task::block_on(join_handle).unwrap()
    }

    Ok(())
}

