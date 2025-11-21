// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`CopyOperation`] data.

use rusqlite::Connection;

use model::CopyOperation;

use crate::{Error, Operation, Result};

/// Creates a new [`CopyOperation`] instance in the database.
pub fn create(conn: &Connection, copy_operation: &mut CopyOperation) -> Result<()> {
    let sql = "
        INSERT INTO copy_operation ( started
                                   , completed
                                   , state
                                   , media_type
                                   , title
                                   , year
                                   , disc
                                   , disc_uuid
                                   , season
                                   , location
                                   , memo
                                   , metadata
                                   , drive_id
                                   , info_log
                                   , copy_log
                                   , host_id
                                   , error
                                   )
             VALUES ( ?1 -- started
                    , ?2 -- completed
                    , ?3 -- state
                    , ?4 -- media_type
                    , ?5 -- title
                    , ?6 -- year
                    , ?7 -- disc
                    , ?8 -- disc_uuid
                    , ?9 -- season
                    , ?10 -- location
                    , ?11 -- memo
                    , ?12 -- metadata
                    , ?13 -- drive_id
                    , ?14 -- info_log
                    , ?15 -- copy_log
                    , ?16 -- host_id
                    , ?17 -- error
                    ) 
          RETURNING id
    ";

    let mut stmt = conn.prepare(sql)
        .map_err(|error| Error::Db { 
            operation: Operation::Prepare,
            error,
        })?;

    let (state, error) = conv::operation_state_to_sql(&copy_operation.state);

    let params = rusqlite::params![
        copy_operation.started.timestamp(),
        copy_operation.completed.timestamp(),
        state,
        conv::media_type_to_sql(&copy_operation.media_type),
        copy_operation.title,
        copy_operation.year,
        copy_operation.disc,
        copy_operation.disc_uuid,
        copy_operation.season,
        copy_operation.location,
        copy_operation.memo,
        copy_operation.metadata.as_bytes(),
        copy_operation.drive.id,
        copy_operation.info_log.as_bytes(),
        copy_operation.copy_log.as_bytes(),
        copy_operation.host.id,
        error,
    ];

    let id = stmt.query_row(params, |r| r.get::<_, u32>(0)).map_err(|error| Error::Db { 
        operation: Operation::Query,
        error,
    })?;

    copy_operation.id = id;

    tracing::info!(id=id, "create copy_operation entry");
    Ok(())
}

/// Creates the database table for storing copy operation data if it does not exist.
pub(crate) fn create_table(conn: &Connection) -> Result<()> {
    let sql = "
        CREATE TABLE copy_operation (
            id          INTEGER  PRIMARY KEY AUTOINCREMENT,
            started     INTEGER  NOT NULL,
            completed   INTEGER  NOT NULL,
            state       INTEGER  NOT NULL,
            error       TEXT     NOT NULL,
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

// TODO: DOC AND MOVE
mod conv {
    use model::{MediaType, OperationState};

    // TODO: DOC
    pub fn operation_state_to_sql(state: &OperationState) -> (u8, String) {
        match state {
            OperationState::Requested => (0, String::default()),
            OperationState::Running => (1, String::default()),
            OperationState::Completed => (2, String::default()),
            OperationState::Cancelled => (3, String::default()),
            OperationState::Failed { reason } => (4, reason.to_owned()),
        }
    }

    // TODO: DOC
    pub fn media_type_to_sql(media_type: &MediaType) -> u8 {
        match media_type {
            MediaType::Movie => 0,
            MediaType::Show => 1,
        }
    }
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

    // TODO: More Testing
}
