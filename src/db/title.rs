// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Title`] data.

use rusqlite::Connection;

use crate::{Error, Result};
use crate::models::Title;

use super::Operation;
use super::conv;

/// Creates a new [`Title`] instance in the database.
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
    use crate::models::MediaType;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        create_table(&conn).expect("Failed to create table");
        conn
    }

    fn make_title(name: &str) -> Title {
        Title {
            id: 0,
            index: 1,
            media_type: MediaType::Movie,
            title: name.to_owned(),
            year: 2024,
            season: 0,
            episode_number: 0,
            episode_count: 0,
            special_feature: None,
            version: String::new(),
            disc: 1,
            location: "shelf-a".to_owned(),
            memo: String::new(),
            videos: None,
        }
    }

    #[test]
    fn test_create_table() {
        let conn = Connection::open_in_memory().unwrap();
        let result = create_table(&conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_title() {
        let conn = setup_test_db();
        let mut title = make_title("Test Movie");

        create(&conn, &mut title).expect("Failed to create title");

        assert!(title.id > 0);
    }

    #[test]
    fn test_create_title_sets_unique_ids() {
        let conn = setup_test_db();
        let mut title1 = make_title("Movie One");
        let mut title2 = make_title("Movie Two");

        create(&conn, &mut title1).unwrap();
        create(&conn, &mut title2).unwrap();

        assert!(title1.id > 0);
        assert!(title2.id > 0);
        assert_ne!(title1.id, title2.id);
    }

    #[test]
    fn test_create_title_show_with_episode() {
        let conn = setup_test_db();
        let mut title = Title {
            id: 0,
            index: 2,
            media_type: MediaType::Show,
            title: "Test Show".to_owned(),
            year: 2023,
            season: 3,
            episode_number: 7,
            episode_count: 2,
            special_feature: None,
            version: String::new(),
            disc: 1,
            location: "shelf-b".to_owned(),
            memo: "double episode".to_owned(),
            videos: None,
        };

        create(&conn, &mut title).expect("Failed to create title");

        assert!(title.id > 0);
    }

    #[test]
    fn test_create_title_with_version() {
        let conn = setup_test_db();
        let mut title = Title {
            id: 0,
            index: 1,
            media_type: MediaType::Movie,
            title: "Test Movie".to_owned(),
            year: 2020,
            season: 0,
            episode_number: 0,
            episode_count: 0,
            special_feature: None,
            version: "Director's Cut".to_owned(),
            disc: 1,
            location: "shelf-a".to_owned(),
            memo: String::new(),
            videos: None,
        };

        create(&conn, &mut title).expect("Failed to create title");

        assert!(title.id > 0);
    }
}
