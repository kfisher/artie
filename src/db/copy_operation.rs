// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`CopyOperation`] data.

use chrono::{DateTime, Utc};

use rusqlite::Connection;

use makemkv::DiscInfo;

use crate::{Error, Result};
use crate::compress;
use crate::models::{CopyOperation, OperationState};

use super::Operation;
use super::conv;

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

    tracing::trace!(?copy_operation, "create copy_operation entry");
    Ok(())
}

/// Creates the database table for storing copy operation data if it does not exist.
pub fn create_table(conn: &Connection) -> Result<()> {
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

/// Set the copy log of the copy operation update the database record.
pub fn set_copy_log(
    conn: &Connection,
    copy_operation: &mut CopyOperation,
    copy_log: &str,
) -> Result<()> {
    let sql = "
        UPDATE copy_operation
           SET copy_log=?1
         WHERE id=?2
    ";

    let bytes = compress::compress(copy_log)?;

    let _ = conn.execute(sql, (bytes, copy_operation.id)).map_err(|error| Error::Db {
        operation: Operation::Execute,
        error,
    })?;

    copy_operation.copy_log = copy_log.to_owned();

    tracing::trace!(id=copy_operation.id, "set copy_operation copy log");
    Ok(())
}

/// Set the info log of the copy operation update the database record.
pub fn set_info_log(
    conn: &Connection,
    copy_operation: &mut CopyOperation,
    info_log: &str,
) -> Result<()> {
    let sql = "
        UPDATE copy_operation
           SET info_log=?1
         WHERE id=?2
    ";

    let bytes = compress::compress(info_log)?;

    let _ = conn.execute(sql, (bytes, copy_operation.id)).map_err(|error| Error::Db {
        operation: Operation::Execute,
        error,
    })?;

    tracing::trace!(id=copy_operation.id, "set copy_operation info log");
    Ok(())
}

/// Set the info log of the copy operation update the database record.
pub fn set_metadata(
    conn: &Connection,
    copy_operation: &mut CopyOperation,
    disc_info: &DiscInfo,
) -> Result<()> {
    let sql = "
        UPDATE copy_operation
           SET metadata=?1
         WHERE id=?2
    ";

    let json = disc_info.as_json()
        .map_err(|error| Error::MakeMKV { error })?;

    // The metadata isn't really needed after the copy operation and is kept for information
    // purposes only. Therefore compress the data.
    let bytes = compress::compress(&json)?;

    let _ = conn.execute(sql, (bytes, copy_operation.id)).map_err(|error| Error::Db {
        operation: Operation::Execute,
        error,
    })?;

    tracing::trace!(id=copy_operation.id, "set copy_operation metadata");
    Ok(())
}

/// Sets the state of the copy operation and update the database record.
/// 
/// This will also set the completed and error fields if applicable based of the state.
pub fn set_state(
    conn: &Connection,
    copy_operation: &mut CopyOperation,
    operation_state: OperationState,
) -> Result<()> {
    let sql = "
        UPDATE copy_operation
           SET state=?1,
               completed=?2,
               error=?3
         WHERE id=?4
    ";

    let (state, error) = conv::operation_state_to_sql(&operation_state);

    let completed = match operation_state {
        OperationState::Completed | OperationState::Cancelled | OperationState::Failed { .. } => {
            Utc::now()
        },
        _ => DateTime::<Utc>::default(),
    };

    let _ = conn.execute(sql, (state, completed.timestamp(), error, copy_operation.id))
        .map_err(|error| Error::Db {
            operation: Operation::Execute,
            error,
        })?;

    copy_operation.state = operation_state;
    copy_operation.completed = completed;

    tracing::trace!(id=copy_operation.id, "set copy_operation state, completed, error");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::models::{OperationState, Reference};

    fn setup_test_db() -> (Connection, u32, u32) {
        let conn = Connection::open_in_memory()
            .expect("Failed to create in-memory database");
        super::super::host::create_table(&conn)
            .expect("Failed to create host table");
        super::super::optical_drive::create_table(&conn)
            .expect("Failed to create optical_drive table");
        create_table(&conn).expect("Failed to create copy_operation table");
        let host = super::super::host::create(&conn, "testhost")
            .expect("Failed to create host");
        let drive = super::super::optical_drive::create(&conn, "SN-TEST-001")
            .expect("Failed to create drive");
        (conn, host.id, drive.id)
    }

    fn make_copy_operation(host_id: u32, drive_id: u32) -> CopyOperation {
        CopyOperation {
            host: Reference { id: host_id, value: None },
            drive: Reference { id: drive_id, value: None },
            ..CopyOperation::default()
        }
    }

    #[test]
    fn test_create_table() {
        let conn = Connection::open_in_memory().unwrap();
        let result = create_table(&conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_copy_operation() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op = make_copy_operation(host_id, drive_id);

        create(&conn, &mut op).expect("Failed to create copy operation");

        assert!(op.id > 0);
    }

    #[test]
    fn test_create_copy_operation_sets_unique_ids() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op1 = make_copy_operation(host_id, drive_id);
        let mut op2 = make_copy_operation(host_id, drive_id);

        create(&conn, &mut op1).unwrap();
        create(&conn, &mut op2).unwrap();

        assert_ne!(op1.id, op2.id);
    }

    #[test]
    fn test_set_copy_log() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op = make_copy_operation(host_id, drive_id);
        create(&conn, &mut op).unwrap();

        set_copy_log(&conn, &mut op, "copy log output").expect("Failed to set copy log");

        assert_eq!(op.copy_log, "copy log output");
    }

    #[test]
    fn test_set_info_log() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op = make_copy_operation(host_id, drive_id);
        create(&conn, &mut op).unwrap();

        let result = set_info_log(&conn, &mut op, "info log output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_state_running() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op = make_copy_operation(host_id, drive_id);
        create(&conn, &mut op).unwrap();

        set_state(&conn, &mut op, OperationState::Running).expect("Failed to set state");

        assert!(matches!(op.state, OperationState::Running));
    }

    #[test]
    fn test_set_state_completed() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op = make_copy_operation(host_id, drive_id);
        create(&conn, &mut op).unwrap();

        set_state(&conn, &mut op, OperationState::Completed).expect("Failed to set state");

        assert!(matches!(op.state, OperationState::Completed));
        assert!(op.completed.timestamp() > 0);
    }

    #[test]
    fn test_set_state_cancelled() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op = make_copy_operation(host_id, drive_id);
        create(&conn, &mut op).unwrap();

        set_state(&conn, &mut op, OperationState::Cancelled).expect("Failed to set state");

        assert!(matches!(op.state, OperationState::Cancelled));
        assert!(op.completed.timestamp() > 0);
    }

    #[test]
    fn test_set_state_failed() {
        let (conn, host_id, drive_id) = setup_test_db();
        let mut op = make_copy_operation(host_id, drive_id);
        create(&conn, &mut op).unwrap();

        set_state(&conn, &mut op, OperationState::Failed { reason: "disk error".to_owned() })
            .expect("Failed to set state");

        assert!(matches!(op.state, OperationState::Failed { .. }));
        assert!(op.completed.timestamp() > 0);
    }
}

