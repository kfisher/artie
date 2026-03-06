// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Title`] data.

use rusqlite::Connection;

use crate::{Error, Result};
use crate::models::Title;

use super::Operation;
use super::conv;

// TODO
pub fn create(conn: &Connection, title: &mut Title) -> Result<()> {
    let sql = "
        INSERT INTO title ( title_index
                          , media_type
                          , title
                          , year
                          , season
                          , episode_number
                          , episode_count
                          , special_feature_kind
                          , special_feature_name
                          , version
                          , disc
                          , location
                          , memo
                          )
             VALUES ( ?1 -- title_index
                    , ?2 -- media_type
                    , ?3 -- title
                    , ?4 -- year
                    , ?5 -- season
                    , ?6 -- episode_number
                    , ?7 -- episode_count
                    , ?8 -- special_feature_kind
                    , ?9 -- special_feature_name
                    , ?10 -- version
                    , ?11 -- disc
                    , ?12 -- location
                    , ?13 -- memo
                    )
          RETURNING id
    ";

    let mut stmt = conn.prepare(sql).map_err(|error| Error::Db { 
            operation: Operation::Prepare,
            error,
        })?;

    let (sf_kind, sf_name) = conv::special_feature_to_sql(&title.special_feature);

    let params = rusqlite::params![
        title.index,
        conv::media_type_to_sql(&title.media_type),
        title.title,
        title.year,
        title.season,
        title.episode_number,
        title.episode_count,
        sf_kind,
        sf_name,
        title.version,
        title.disc,
        title.location,
        title.memo,
    ];

    let id = stmt.query_row(params, |r| r.get::<_, u32>(0)).map_err(|error| Error::Db { 
        operation: Operation::Query,
        error,
    })?;

    title.id = id;

    tracing::trace!(?title, "create title entry");
    Ok(())
}

/// Creates the database table for storing title data if it does not exist.
pub fn create_table(conn: &Connection) -> Result<()> {
    let sql = "
        CREATE TABLE title (
            id                    INTEGER  PRIMARY KEY AUTOINCREMENT,
            title_index           INTEGER  NOT NULL,
            media_type            INTEGER  NOT NULL,
            title                 TEXT     NOT NULL,
            year                  INTEGER  NOT NULL,
            season                INTEGER  NOT NULL,
            episode_number        INTEGER  NOT NULL,
            episode_count         INTEGER  NOT NULL,
            special_feature_kind  INTEGER  NOT NULL,
            special_feature_name  TEXT     NOT NULL,
            version               TEXT     NOT NULL,
            disc                  INTEGER  NOT NULL,
            location              TEXT     NOT NULL,
            memo                  TEXT     NOT NULL
        ) STRICT
    ";

    let _ = conn.execute(sql, ()).map_err(|error| Error::Db { 
            operation: Operation::Execute,
            error,
        })?;

    tracing::info!("create title table");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    //-] /// Helper function to create an in-memory database with the title table
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
