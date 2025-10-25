// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`OpticalDrive`] data.

use rusqlite::{Connection, OptionalExtension};

use model::OpticalDrive;

use crate::{Error, Operation, Result};


/// Creates a new [`OpticalDrive`] instance in the database.
pub fn create(conn: &Connection, serial_number: &str) -> Result<OpticalDrive> {
    if serial_number.trim().is_empty() {
        return Err(Error::EmptyString { arg: String::from("serial_number") });
    }

    let sql = "
        INSERT INTO optical_drive (serial_number)
             VALUES (?1) 
          RETURNING id
    ";

    let mut stmt = conn.prepare(sql).map_err(|error| Error::Db { 
            operation: Operation::Prepare,
            error,
        })?;

    let id = stmt.query_row((serial_number,), |r| r.get::<_, u32>(0)).map_err(|error| Error::Db { 
        operation: Operation::Query,
        error,
    })?;

    let drive = OpticalDrive {
        id, 
        serial_number: serial_number.to_owned(),
    };

    tracing::info!(id=id, serial_number=serial_number, "create optical_drive entry");
    Ok(drive)
}

/// Gets an [`OpticalDrive`] from the database using its serial number if it exists.
pub fn get_by_serial_number(
    conn: &Connection,
    serial_number: &str
) -> Result<Option<OpticalDrive>> {
    let sql = "
        SELECT optical_drive.id, optical_drive.serial_number
          FROM optical_drive
         WHERE serial_number=:serial_number
    ";

    let mut stmt = conn.prepare(sql).map_err(|error| Error::Db { 
        operation: Operation::Prepare,
        error,
    })?;

    let drive = stmt.query_one(
        &[(":serial_number", serial_number)], 
        |r| Ok(OpticalDrive {
            id: r.get::<_, u32>(0)?,
            serial_number: r.get::<_, String>(1)?
        })
    ).optional().map_err(|error| Error::Db { 
        operation: Operation::Query,
        error,
    })?;

    Ok(drive)
}

/// Gets an [`OpticalDrive`] from the database using its serial number if it exists or creates a 
/// new instance if it does not exist.
pub fn get_or_create(conn: &Connection, serial_number: &str) -> Result<OpticalDrive> {
    match get_by_serial_number(conn, serial_number)? {
        Some(drive) => Ok(drive),
        None => create(conn, serial_number),
    }
}

/// Creates the database table for storing optical drive data if it does not exist.
pub(crate) fn create_table(conn: &Connection) -> Result<()> {
    let sql = "
        CREATE TABLE optical_drive (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            serial_number TEXT    UNIQUE NOT NULL
        ) STRICT
    ";

    let _ = conn.execute(sql, ()).map_err(|error| Error::Db { 
            operation: Operation::Execute,
            error,
        })?;

    tracing::info!("create optical_drive table");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Helper function to create an in-memory database with the optical_drive table
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        create_table(&conn).expect("Failed to create table");
        conn
    }

    #[test]
    fn test_create_table() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Should succeed
        let result = create_table(&conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_optical_drive() {
        let conn = setup_test_db();
        let serial = "SN12345";
        
        let drive = create(&conn, serial).expect("Failed to create optical drive");
        
        assert_eq!(drive.serial_number, serial);
        assert!(drive.id > 0);
    }

    #[test]
    fn test_create_duplicate_serial_number_fails() {
        let conn = setup_test_db();
        let serial = "SN12345";
        
        // First insert should succeed
        let result = create(&conn, serial);
        assert!(result.is_ok());
        
        // Second insert with same serial should fail due to UNIQUE constraint
        let result = create(&conn, serial);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_by_serial_number_existing() {
        let conn = setup_test_db();
        let serial = "SN12345";
        
        // Create a drive
        let created = create(&conn, serial).unwrap();
        
        // Retrieve it
        let retrieved = get_by_serial_number(&conn, serial)
            .expect("Failed to get optical drive")
            .expect("Drive should exist");
        
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.serial_number, created.serial_number);
    }

    #[test]
    fn test_get_by_serial_number_not_found() {
        let conn = setup_test_db();
        
        // Try to get a non-existent drive
        let result = get_by_serial_number(&conn, "NONEXISTENT")
            .expect("Query should succeed");
        
        assert!(result.is_none());
    }

    #[test]
    fn test_get_or_create_when_exists() {
        let conn = setup_test_db();
        let serial = "SN12345";
        
        // Create initial drive
        let original = create(&conn, serial).unwrap();
        
        // get_or_create should return the existing one
        let retrieved = get_or_create(&conn, serial).unwrap();
        
        assert_eq!(retrieved.id, original.id);
        assert_eq!(retrieved.serial_number, original.serial_number);
    }

    #[test]
    fn test_get_or_create_when_not_exists() {
        let conn = setup_test_db();
        let serial = "SN12345";
        
        // get_or_create should create a new drive
        let drive = get_or_create(&conn, serial).unwrap();
        
        assert_eq!(drive.serial_number, serial);
        assert!(drive.id > 0);
        
        // Verify it was actually created
        let retrieved = get_by_serial_number(&conn, serial)
            .unwrap()
            .expect("Drive should exist");
        
        assert_eq!(retrieved.id, drive.id);
    }

    #[test]
    fn test_get_or_create_idempotent() {
        let conn = setup_test_db();
        let serial = "SN12345";
        
        // Call multiple times
        let drive1 = get_or_create(&conn, serial).unwrap();
        let drive2 = get_or_create(&conn, serial).unwrap();
        let drive3 = get_or_create(&conn, serial).unwrap();
        
        // All should return the same drive
        assert_eq!(drive1.id, drive2.id);
        assert_eq!(drive2.id, drive3.id);
        assert_eq!(drive1.serial_number, drive2.serial_number);
    }

    #[test]
    fn test_empty_serial_number() {
        let conn = setup_test_db();
        
        let result = create(&conn, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_serial_number_with_special_characters() {
        let conn = setup_test_db();
        
        let serials = vec![
            "SN-123-456",
            "SN_123_456",
            "SN@123#456",
            "SN with spaces",
            "SN'with'quotes",
            "SN\"with\"doublequotes",
        ];
        
        for serial in serials {
            let drive = create(&conn, serial)
                .expect(&format!("Failed to create drive with serial: {}", serial));
            assert_eq!(drive.serial_number, serial);
            
            let retrieved = get_by_serial_number(&conn, serial)
                .unwrap()
                .expect("Drive should exist");
            assert_eq!(retrieved.serial_number, serial);
        }
    }

    #[test]
    fn test_transaction_rollback() {
        let mut conn = setup_test_db();
        
        // Start a transaction
        let tx = conn.transaction().unwrap();
        
        create(&tx, "SN-TX-001").unwrap();
        
        // Rollback the transaction
        tx.rollback().unwrap();
        
        // The drive should not exist
        let result = get_by_serial_number(&conn, "SN-TX-001").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_transaction_commit() {
        let mut conn = setup_test_db();
        
        // Start a transaction
        let tx = conn.transaction().unwrap();
        
        let drive = create(&tx, "SN-TX-002").unwrap();
        
        // Commit the transaction
        tx.commit().unwrap();
        
        // The drive should exist
        let result = get_by_serial_number(&conn, "SN-TX-002")
            .unwrap()
            .expect("Drive should exist");
        assert_eq!(result.id, drive.id);
    }
}
