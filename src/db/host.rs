// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Host`] data.

use rusqlite::{Connection, OptionalExtension};

use crate::{Error, Result};
use crate::models::Host;

use super::Operation;

/// Gets an [`Host`] from the database using its hostname if it exists or creates a new instance
/// if it does not exist.
pub fn get_or_create(conn: &Connection, hostname: &str) -> Result<Host> {
    match get_by_serial_number(conn, hostname)? {
        Some(drive) => Ok(drive),
        None => create(conn, hostname),
    }
}

/// Creates a new [`Host`] instance in the database.
pub fn create(conn: &Connection, hostname: &str) -> Result<Host> {
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

    tracing::trace!(?host, "create host entry");
    Ok(host)
}

/// Gets an [`Host`] from the database using its hostname if it exists.
pub fn get_by_serial_number(conn: &Connection, hostname: &str) -> Result<Option<Host>> {
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
pub fn create_table(conn: &Connection) -> Result<()> {
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

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        create_table(&conn).expect("Failed to create table");
        conn
    }

    #[test]
    fn test_create_table() {
        let conn = Connection::open_in_memory().unwrap();
        let result = create_table(&conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_host() {
        let conn = setup_test_db();
        let hostname = "myhost.local";

        let host = create(&conn, hostname).expect("Failed to create host");

        assert_eq!(host.hostname, hostname);
        assert!(host.id > 0);
    }

    #[test]
    fn test_create_empty_hostname_fails() {
        let conn = setup_test_db();

        let result = create(&conn, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_whitespace_hostname_fails() {
        let conn = setup_test_db();

        let result = create(&conn, "   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_duplicate_hostname_fails() {
        let conn = setup_test_db();
        let hostname = "myhost.local";

        let result = create(&conn, hostname);
        assert!(result.is_ok());

        let result = create(&conn, hostname);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_by_hostname_existing() {
        let conn = setup_test_db();
        let hostname = "myhost.local";

        let created = create(&conn, hostname).unwrap();

        let retrieved = get_by_serial_number(&conn, hostname)
            .expect("Query should succeed")
            .expect("Host should exist");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.hostname, created.hostname);
    }

    #[test]
    fn test_get_by_hostname_not_found() {
        let conn = setup_test_db();

        let result = get_by_serial_number(&conn, "nonexistent.local")
            .expect("Query should succeed");

        assert!(result.is_none());
    }

    #[test]
    fn test_get_or_create_when_exists() {
        let conn = setup_test_db();
        let hostname = "myhost.local";

        let original = create(&conn, hostname).unwrap();
        let retrieved = get_or_create(&conn, hostname).unwrap();

        assert_eq!(retrieved.id, original.id);
        assert_eq!(retrieved.hostname, original.hostname);
    }

    #[test]
    fn test_get_or_create_when_not_exists() {
        let conn = setup_test_db();
        let hostname = "myhost.local";

        let host = get_or_create(&conn, hostname).unwrap();

        assert_eq!(host.hostname, hostname);
        assert!(host.id > 0);

        let retrieved = get_by_serial_number(&conn, hostname)
            .unwrap()
            .expect("Host should exist");
        assert_eq!(retrieved.id, host.id);
    }

    #[test]
    fn test_get_or_create_idempotent() {
        let conn = setup_test_db();
        let hostname = "myhost.local";

        let host1 = get_or_create(&conn, hostname).unwrap();
        let host2 = get_or_create(&conn, hostname).unwrap();
        let host3 = get_or_create(&conn, hostname).unwrap();

        assert_eq!(host1.id, host2.id);
        assert_eq!(host2.id, host3.id);
        assert_eq!(host1.hostname, host2.hostname);
    }

    #[test]
    fn test_hostname_with_special_characters() {
        let conn = setup_test_db();

        let hostnames = vec![
            "host-01.local",
            "host_02.local",
            "host with spaces",
            "host'with'quotes",
            "host\"with\"doublequotes",
        ];

        for hostname in hostnames {
            let host = create(&conn, hostname)
                .unwrap_or_else(|_| panic!("Failed to create host with hostname: {}", hostname));
            assert_eq!(host.hostname, hostname);

            let retrieved = get_by_serial_number(&conn, hostname)
                .unwrap()
                .expect("Host should exist");
            assert_eq!(retrieved.hostname, hostname);
        }
    }
}
