// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Video`] data.

use rusqlite::Connection;

use crate::{Error, Result};
use crate::error;
use crate::models::{Video, VideoSource};

use super::Operation;
use super::conv;

pub fn create(conn: &Connection, video: &mut Video) -> Result<()> {
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
        serde_json::to_string(&video.video_tracks)
            .map_err(error::json_serialize)?,
        serde_json::to_string(&video.audio_tracks)
            .map_err(error::json_serialize)?,
        serde_json::to_string(&video.subtitle_tracks)
            .map_err(error::json_serialize)?,
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
    use std::time::Duration;

    use super::*;

    use rusqlite::Connection;

    use crate::db::copy_operation;
    use crate::db::host;
    use crate::db::optical_drive;
    use crate::db::title;
    use crate::db::transcode_operation;
    use crate::models::{
        ContainerType,
        CopyOperation,
        MediaLocation,
        MediaType,
        Reference,
        VideoSource
    };

    /// Creates all required tables and seed data, returning (conn, copy_operation_id, title_id).
    fn setup_test_db() -> (Connection, u32, u32) {
        let conn = Connection::open_in_memory()
            .expect("Failed to create in-memory database");
        host::create_table(&conn)
            .expect("Failed to create host table");
        optical_drive::create_table(&conn)
            .expect("Failed to create optical_drive table");
        title::create_table(&conn)
            .expect("Failed to create title table");
        copy_operation::create_table(&conn)
            .expect("Failed to create copy_operation table");
        transcode_operation::create_table(&conn)
            .expect("Failed to create transcode_operation table");
        create_table(&conn).expect("Failed to create video table");

        let host = host::create(&conn, "testhost").expect("Failed to create host");
        let drive = optical_drive::create(&conn, "SN-TEST-001").expect("Failed to create drive");

        let mut title = crate::models::Title {
            id: 0,
            index: 1,
            media_type: MediaType::Movie,
            title: "Test Movie".to_owned(),
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
        };
        title::create(&conn, &mut title).expect("Failed to create title");

        let mut copy_op = CopyOperation {
            host: Reference { id: host.id, value: None },
            drive: Reference { id: drive.id, value: None },
            ..CopyOperation::default()
        };
        copy_operation::create(&conn, &mut copy_op).expect("Failed to create copy operation");

        (conn, copy_op.id, title.id)
    }

    fn make_video(copy_op_id: u32, title_id: u32) -> Video {
        Video {
            id: 0,
            location: MediaLocation::Inbox(std::path::PathBuf::from("movies/test.mkv")),
            checksum: blake3::hash(b"test video data"),
            container: ContainerType::MKV,
            video_tracks: vec![],
            audio_tracks: vec![],
            subtitle_tracks: vec![],
            source: VideoSource::CopyOperation(Reference { id: copy_op_id, value: None }),
            title: Reference { id: title_id, value: None },
            duration: Duration::from_secs(7200),
        }
    }

    #[test]
    fn test_create_table() {
        let conn = Connection::open_in_memory().unwrap();
        let result = create_table(&conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_video() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);

        create(&conn, &mut video).expect("Failed to create video");

        assert!(video.id > 0);
    }

    #[test]
    fn test_create_video_sets_unique_ids() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video1 = make_video(copy_op_id, title_id);
        let mut video2 = make_video(copy_op_id, title_id);

        create(&conn, &mut video1).unwrap();
        create(&conn, &mut video2).unwrap();

        assert_ne!(video1.id, video2.id);
    }

    #[test]
    fn test_create_video_library_location() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = Video {
            location: MediaLocation::Library(std::path::PathBuf::from("movies/test.mkv")),
            ..make_video(copy_op_id, title_id)
        };

        create(&conn, &mut video).expect("Failed to create video");

        assert!(video.id > 0);
    }
}
