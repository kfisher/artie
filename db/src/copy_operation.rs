// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`CopyOperation`] data.

use rusqlite::Connection;

use crate::{Error, Operation, Result};

/// Creates the database table for storing copy operation data if it does not exist.
pub(crate) fn create_table(conn: &Connection) -> Result<()> {
    let sql = "
        CREATE TABLE copy_operation (
            id          INTEGER  PRIMARY KEY AUTOINCREMENT,
            started     INTEGER  NOT NULL,
            completed   INTEGER  NOT NULL,
            state       INTEGER  NOT NULL,
            media_type  INTEGER  NOT NULL,
            title       TEXT     NOT NULL,
            year        INTEGER  NOT NULL,
            disc        INTEGER  NOT NULL,
            disc_uuid   TEXT     NOT NULL,
            season      INTEGER  NOT NULL,
            location    TEXT     NOT NULL,
            memo        TEXT     NOT NULL,
            metadata    BLOB     NOT NULL,
            drive_id    INTEGER  NOT NULL,
            info_log    BLOB     NOT NULL,
            copy_log    BLOB     NOT NULL,
            host_id     INTEGER  NOT NULL,
            FOREIGN KEY(drive_id) REFERENCES optical_drive(id),
            FOREIGN KEY(host_id)  REFERENCES host(id)
        ) STRICT
    ";

    let _ = conn.execute(sql, ()).map_err(|error| Error::Db { 
            operation: Operation::Execute,
            error,
        })?;

    tracing::info!("create copy_operation table");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    //-] /// Helper function to create an in-memory database with the copy_operation table
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
}
