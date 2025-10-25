// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`TranscodeOperation`] data.

use rusqlite::Connection;

use crate::{Error, Operation, Result};

/// Creates the database table for storing transcode operation data if it does not exist.
pub(crate) fn create_table(conn: &Connection) -> Result<()> {
    let sql = "
        CREATE TABLE transcode_operation (
            id                    INTEGER  PRIMARY KEY AUTOINCREMENT,
            started               INTEGER  NOT NULL,
            completed             INTEGER  NOT NULL,
            state                 INTEGER  NOT NULL,
            episode_number        INTEGER  NOT NULL,
            episode_count         INTEGER  NOT NULL,
            special_feature_kind  INTEGER  NOT NULL,
            special_feature_name  TEXT     NOT NULL,
            version               TEXT     NOT NULL,
            audio_tracks          BLOB     NOT NULL,
            subtitle_tracks       BLOB     NOT NULL,
            command_log           BLOB     NOT NULL,
            host_id               INTEGER  NOT NULL,
            title_id              INTEGER  NOT NULL,
            FOREIGN KEY(host_id)  REFERENCES host(id),
            FOREIGN KEY(title_id) REFERENCES title(id)
        ) STRICT
    ";

    let _ = conn.execute(sql, ()).map_err(|error| Error::Db { 
            operation: Operation::Execute,
            error,
        })?;

    tracing::info!("create transcode_operation table");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    //-] /// Helper function to create an in-memory database with the transcode_operation table
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
