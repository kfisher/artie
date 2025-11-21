// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Host`] data.

use rusqlite::{Connection, OptionalExtension};

use model::Host;

use crate::{Error, Operation, Result};

/// Gets an [`Host`] from the database using its hostname if it exists or creates a new instance
/// if it does not exist.
pub fn get_or_create(conn: &Connection, hostname: &str) -> Result<Host> {
    match get_by_serial_number(conn, hostname)? {
        Some(drive) => Ok(drive),
        None => create(conn, hostname),
    }
}

/// Creates a new [`Host`] instance in the database.
pub(crate) fn create(conn: &Connection, hostname: &str) -> Result<Host> {
    if hostname.trim().is_empty() {
        return Err(Error::EmptyString { arg: String::from("hostname") });
    }

    let sql = "
        INSERT INTO host (hostname)
             VALUES (?1) 
          RETURNING id
    ";

    let mut stmt = conn.prepare(sql)
        .map_err(|error| Error::Db { 
            operation: Operation::Prepare,
            error,
        })?;

    let id = stmt.query_row((hostname,), |r| r.get::<_, u32>(0))
        .map_err(|error| Error::Db { 
            operation: Operation::Query,
            error,
        })?;

    let host = Host {
        id,
        hostname: hostname.to_owned(),
    };

    tracing::info!(id=id, "create host entry");
    Ok(host)
}

/// Gets an [`Host`] from the database using its hostname if it exists.
pub(crate) fn get_by_serial_number(conn: &Connection, hostname: &str) -> Result<Option<Host>> {
    let sql = "
        SELECT host.id, host.hostname
          FROM host
         WHERE hostname=:hostname
    ";

    let mut stmt = conn.prepare(sql)
        .map_err(|error| Error::Db { 
            operation: Operation::Prepare,
            error,
        })?;

    let host = stmt.query_one(
        &[(":hostname", hostname)], 
        |r| Ok(Host {
            id: r.get::<_, u32>(0)?,
            hostname: r.get::<_, String>(1)?
        })
    ).optional().map_err(|error| Error::Db { 
        operation: Operation::Query,
        error,
    })?;

    Ok(host)
}

/// Creates the database table for storing host data if it does not exist.
pub(crate) fn create_table(conn: &Connection) -> Result<()> {
    let sql = "
        CREATE TABLE host (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            hostname  TEXT    UNIQUE NOT NULL
        ) STRICT
    ";

    let _ = conn.execute(sql, ()).map_err(|error| Error::Db { 
            operation: Operation::Execute,
            error,
        })?;

    tracing::info!("create host table");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    //-] /// Helper function to create an in-memory database with the host table
    //-] fn setup_test_db() -> Connection {
    //-]     let conn = Connection::open_in_memory()
    //-]         .expect("Failed to create in-memory database");
    //-]     create_table(&conn)
    //-]         .expect("Failed to create table");
    //-]     conn
    //-] }

    #[test]
    fn test_create_table() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Should succeed
        let result = create_table(&conn);
        assert!(result.is_ok());
    }

    // TODO: More Testing
}
