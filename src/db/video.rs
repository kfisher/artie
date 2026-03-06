// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Video`] data.

use rusqlite::Connection;

use crate::{Error, Result};
use crate::models::{Video, VideoSource};

use super::Operation;
use super::conv;



pub fn create(conn: &Connection, video: &mut Video) -> Result<()> {
        // id: 0,
        // location: todo!(),
        // checksum: todo!(),
        // container: todo!(),
        // video_tracks: todo!(),
        // audio_tracks: todo!(),
        // subtitle_tracks: todo!(),
        // source: todo!(),
        // title: todo!(),
        // duration: todo!(),

    let sql = "
        INSERT INTO video ( location_area
                          , location_path
                          , checksum
                          , container
                          , video_tracks
                          , audio_tracks
                          , subtitle_tracks
                          , copy_operation_id
                          , transcode_operation_id
                          , title_id
                          , duration
                          )
             VALUES ( ?1        -- location_area
                    , ?2        -- location_path
                    , ?3        -- checksum
                    , ?4        -- container
                    , jsonb(?5) -- video_tracks
                    , jsonb(?6) -- audio_tracks
                    , jsonb(?7) -- subtitle_tracks
                    , ?8        -- copy_operation_id
                    , ?9        -- transcode_operation_id
                    , ?10       -- title_id
                    , ?11       -- duration
                    )
          RETURNING id
    ";

    let (loc_area, loc_path) = conv::media_location_to_sql(&video.location);

    let mut stmt = conn.prepare(sql).map_err(|error| Error::Db { 
            operation: Operation::Prepare,
            error,
        })?;
    
    let (copy_operation, transcode_operation) = match &video.source {
        VideoSource::CopyOperation(reference) => (Some(reference.id), None),
        VideoSource::TranscodeOperation(reference) => (None, Some(reference.id)),
    };

    let checksum = video.checksum.to_hex();

    let params = rusqlite::params![
        loc_area,
        loc_path,
        checksum.as_str(),
        conv::container_type_to_sql(&video.container),
        serde_json::to_string(&video.video_tracks).unwrap(), // FIXME: ERROR
        serde_json::to_string(&video.audio_tracks).unwrap(), // FIXME: ERROR
        serde_json::to_string(&video.subtitle_tracks).unwrap(), // FIXME: ERROR
        copy_operation,
        transcode_operation,
        video.title.id,
        video.duration.as_secs(),
    ];

    let id = stmt.query_row(params, |r| r.get::<_, u32>(0)).map_err(|error| Error::Db { 
        operation: Operation::Query,
        error,
    })?;

    video.id = id;

    tracing::trace!(?video, "create video entry");
    Ok(())
}

/// Creates the database table for storing video data if it does not exist.
pub fn create_table(conn: &Connection) -> Result<()> {
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
