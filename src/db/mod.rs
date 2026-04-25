// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles interactions with the application database.
//!
//! To perform database operations, open a connection to the database using the [`connect`]
//! function to request a connection from the database actor. The same connection cannot be used
//! accross threads. Therefore, a new connection should be established within each task.
//!
//! Before opening a connection, [`init`] must be called to perform initialization. This happens
//! during application startup.

mod conv;
pub mod copy_operation;
pub mod host;
pub mod optical_drive;
pub mod title;
pub mod transaction;
pub mod transcode_operation;
pub mod video;

use std::path::PathBuf;

use rusqlite::Connection;

use tokio::sync::oneshot;

use crate::{Error, Result};
use crate::actor::{self, Response};
use crate::bus;
use crate::path;

/// The name of the SQLite database file.
const DATABASE_NAME: &str = "artie.db";


/// Handle used to communicate with the database actor.
pub type Handle = actor::Handle<Message>;

/// Messages used to send requests to the database.
#[derive(Debug)]
pub enum Message {
    /// Open a connection to the database.
    Connect {
        response: Response<Connection>,
    }
}

/// Open a connection to the database.
///
/// `bus`:  Handle for sending messages to the database actor.
///
/// # Errors
///
/// [`Error::Database`] if the connection fails.
///
/// [`Error::ChannelSend`] if the request could not be sent to the database actor.
///
/// [`Error::ResponseRecv`] if the there was an error while waiting for the response from the
/// database actor to the request.
pub async fn connect(bus: &bus::Handle) -> Result<Connection> {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Connect { response: tx };
    bus.send(msg).await?;
    rx.await?
}

/// Initialize the database.
///
/// This will create the actor, spawn the task to process requests, and perform any required
/// database migrations.
///
/// # Errors
///
/// [`Error::Database`] if a database operation fails while determining the need for and running
/// migrations.
pub fn init() -> Result<Handle> {
    let msg_processor = MessageProcessor::new(path::data_path(DATABASE_NAME));

    let mut run_migrations = true;

    // If the path already exists, assume the database is already initialized. This will only
    // work during initial development. Beyond that, we'll need to track versions.
    if msg_processor.db_path.is_file() {
        run_migrations = false;
    }

    let conn = msg_processor.connect()?;

    if run_migrations {
        migration_0(&conn)?;
    }

    tracing::info!("database initialized");

    Ok(actor::create_and_run("database", msg_processor))
}

/// Processes messages sent to the database actor.
struct MessageProcessor {
    /// The path to the SQLite file.
    db_path: PathBuf,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    ///
    /// # Args
    ///
    /// `db_path`:  The path to the SQLite file.
    fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Open a connection to the database.
    fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        Ok(conn)
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Connect { response } => {
                response.send(self.connect())
                    .inspect_err(|_| send_error_trace("Connect"))
                    .map_err(|_| Error::ResponseSend)
            },
        }
    }
}

/// Initializes the database schema.
///
/// # Args
///
/// `conn`:  The connection to the database.
///
/// # Errors
///
/// [`Error::Database`] if any of the database operations fail.
fn migration_0(conn: &Connection) -> Result<()> {

    // NOTE: Order is important here in order for foreign key references to be configured
    //       correctly.

    host::create_table(conn)?;
    optical_drive::create_table(conn)?;
    title::create_table(conn)?;

    copy_operation::create_table(conn)?;
    transcode_operation::create_table(conn)?;

    video::create_table(conn)?;

    tracing::info!("completed migration 0");

    Ok(())
}

/// Log an error due to failure to send a response.
/// 
/// # Args
///
/// `request`:  The name of the request the response was being sent for.
fn send_error_trace(request: &str) {
    tracing::error!("failed to send {} response", request);
}
