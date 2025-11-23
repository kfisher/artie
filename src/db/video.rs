// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Video`] data.

use rusqlite::Connection;

use crate::{Error, Result};

use super::Operation;

/// Creates the database table for storing video data if it does not exist.
pub(crate) fn create_table(conn: &Connection) -> Result<()> {
    let sql = "
        CREATE TABLE video (
            id                     INTEGER  PRIMARY KEY AUTOINCREMENT,
            location_area          INTEGER  NOT NULL,
            location_path          TEXT     NOT NULL,
            checksum               TEXT     NOT NULL,
            container              INTEGER  NOT NULL,
            video_tracks           BLOB     NOT NULL,
            audio_tracks           BLOB     NOT NULL,
            subtitle_tracks        BLOB     NOT NULL,
            copy_operation_id      INTEGER,
            transcode_operation_id INTEGER,
            title_id               INTEGER  NOT NULL,
            duration               INTEGER  NOT NULL,
            FOREIGN KEY(copy_operation_id)      REFERENCES copy_operation(id),
            FOREIGN KEY(transcode_operation_id) REFERENCES transcode_operation(id),
            FOREIGN KEY(title_id)               REFERENCES title(id)
        ) STRICT
    ";

    let _ = conn.execute(sql, ()).map_err(|error| Error::Db { 
            operation: Operation::Execute,
            error,
        })?;

    tracing::info!("create video table");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    //-] /// Helper function to create an in-memory database with the video table
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
