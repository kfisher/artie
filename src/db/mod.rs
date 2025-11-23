// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the database interface.

pub mod copy_operation;
pub mod host;
pub mod optical_drive;
pub mod title;
pub mod transcode_operation;
pub mod video;

use std::path::PathBuf;

use rusqlite::Connection;

use crate::{Error, Result};

/// Specifies the different database operations.
///
/// This is used to identify the action that was being performed when an error occurs.
#[derive(Debug)]
pub enum Operation {
    Execute,
    Prepare,
    Query,
}

/// Database configuration settings.
pub struct Settings {
    /// The path to the database file.
    ///
    /// A `None` value will result in the database being created in-memory.
    pub path: Option<PathBuf>,
}

/// Open a connection to the database.
pub fn connect(settings: &Settings) -> Result<Connection> {
    match settings.path.as_ref() {
        Some(path) => Connection::open(path),
        None => Connection::open_in_memory(),
    }.map_err(|error| Error::Connect {
        path: settings.path.clone(),
        error 
    })
}

/// Initializes the database. 
pub fn init(settings: &Settings) -> Result<()> {
    let span = tracing::info_span!("db_init");
    let _guard = span.enter();

    let mut run_migrations = true;

    // If the path already exists, assume the database is already initialized. This will only work
    // during initial development. Beyond that, we'll need to track versions.
    if let Some(path) = settings.path.as_ref() && path.is_file() {
        run_migrations = false;
    }

    let conn = connect(settings)?;

    if run_migrations {
        migration_0(&conn)?;
    }

    tracing::info!("database initialized");
    Ok(())
}

/// Initializes the database schema.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let settings = Settings {
            path: None,
        };

        let result = init(&settings);
        assert!(result.is_ok());
    }
}
