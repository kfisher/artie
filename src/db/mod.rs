// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the database interface.

pub mod conv;
pub mod copy_operation;
pub mod host;
pub mod optical_drive;
pub mod title;
pub mod transcode_operation;
pub mod video;

use std::path::PathBuf;

use rusqlite::Connection;

use crate::{Error, Result};
use crate::fs::FileSystem;

/// Specifies the different database operations.
///
/// This is used to identify the action that was being performed when an error occurs.
#[derive(Debug)]
pub enum Operation {
    Execute,
    Prepare,
    Query,
    Transaction,
}

/// Provides the interface for connecting to and initializing the database.
#[derive(Debug, Clone)]
pub struct Database {
    /// Database connection and configuration settings.
    settings: Settings,
}

impl Database {
    /// Create a new [`Database`] instance.
    fn new(fs: &FileSystem) -> Self {
        Self {
            settings: Settings {
                path: Some(fs.data_path(DATABASE_NAME)),
            }
        }
    }

    /// Initializes the database.
    fn init(&self) -> Result<()> {
        let mut run_migrations = true;

        // If the path already exists, assume the database is already initialized. This will only
        // work during initial development. Beyond that, we'll need to track versions.
        if let Some(path) = self.settings.path.as_ref() && path.is_file() {
            run_migrations = false;
        }

        let conn = self.connect()?;

        if run_migrations {
            migration_0(&conn)?;
        }

        tracing::info!("database initialized");
        Ok(())
    }

    /// Open a connection to the database.
    pub fn connect(&self) -> Result<Connection> {
        match self.settings.path.as_ref() {
            Some(path) => Connection::open(path),
            None => Connection::open_in_memory(),
        }.map_err(|error| Error::Connect {
            path: self.settings.path.clone(),
            error 
        })
    }
}

/// Database configuration settings.
#[derive(Debug, Default, Clone)]
pub struct Settings {
    /// The path to the database file.
    ///
    /// A `None` value will result in the database being created in-memory.
    pub path: Option<PathBuf>,
}

/// Create a new [`Database`] instance and run required migrations.
pub fn init(fs: &FileSystem) -> Result<Database> {
    let db = Database::new(fs);
    db.init()?;
    Ok(db)
}

/// The name of the SQLite database file.
const DATABASE_NAME: &str = "artie.db";

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

pub mod transaction {
    use rusqlite::{Connection, Transaction};

    use crate::{Error, Result};

    use super::Operation;

    /// Starts a new database transaction.
    pub fn start(conn: &mut Connection) -> Result<Transaction<'_>> {
        conn.transaction()
            .map_err(|error| Error::Db { operation: Operation::Transaction, error })
    }

    /// Commits the transaction.
    pub fn commit(transaction: Transaction) -> Result<()> {
        transaction.commit()
            .map_err(|error| Error::Db { operation: Operation::Transaction, error })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let db = Database {
            settings: Settings::default(),
        };
        let result = db.init();
        assert!(result.is_ok());
    }
}
