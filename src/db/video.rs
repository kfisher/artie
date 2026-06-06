// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Database operations for [`Video`] data.

use std::path::PathBuf;
use std::time::Duration;

use rusqlite::Connection;

use crate::{Error, Result};
use crate::models::{AudioTrack, MediaLocation, Reference, SubtitleTrack, Title, Video, VideoSource, VideoTrack};

use super::conv;

/// Creates a new video record in the database.
///
/// # Args
///
/// `conn`:  The connection to the database.
///
/// `title`:   The video data to create the record from. If successful, the id field will be set.
///
/// # Errors
///
/// [`crate::Error::Database`] raised if the database operation fails.
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

    let mut stmt = conn.prepare(sql)?;

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
        serde_json::to_string(&video.video_tracks)?,
        serde_json::to_string(&video.audio_tracks)?,
        serde_json::to_string(&video.subtitle_tracks)?,
        copy_operation,
        transcode_operation,
        video.title.id,
        video.duration.as_secs(),
    ];

    let id = stmt.query_row(params, |r| r.get::<_, u32>(0))?;

    video.id = id;

    tracing::trace!(?video, "create video entry");
    Ok(())
}

/// Returns a list of all videos currently in the media inbox.
///
/// # Args
///
/// `conn`:  The connection to the database.
///
/// # Errors
///
/// [`crate::Error::Blake`] ...
///
/// [`crate::Error::Database`] raised if the database operation fails.
pub fn inbox_videos(conn: &Connection) -> Result<Vec<Video>> {
    let sql = "
        SELECT v.id
             , v.location_path
             , v.checksum
             , v.container
             , json(v.video_tracks)
             , json(v.audio_tracks)
             , json(v.subtitle_tracks)
             , v.copy_operation_id
             , v.transcode_operation_id
             , v.duration
             , t.id
             , t.title_index
             , t.media_type
             , t.title
             , t.year
             , t.season
             , t.episode_number
             , t.episode_count
             , t.special_feature_kind
             , t.special_feature_name
             , t.version
             , t.disc
             , t.location
             , t.memo
          FROM video v
          JOIN title t ON v.title_id = t.id
         WHERE v.location_area = 1
    ";

    let mut stmt = conn.prepare(sql)?;

    stmt.query_map([], |row| {
        Ok((
            row.get::<_, u32>(0)?,         // v.id
            row.get::<_, String>(1)?,      // v.location_path
            row.get::<_, String>(2)?,      // v.checksum
            row.get::<_, u8>(3)?,          // v.container
            row.get::<_, String>(4)?,      // v.video_tracks
            row.get::<_, String>(5)?,      // v.audio_tracks
            row.get::<_, String>(6)?,      // v.subtitle_tracks
            row.get::<_, Option<u32>>(7)?, // v.copy_operation_id
            row.get::<_, Option<u32>>(8)?, // v.transcode_operation_id
            row.get::<_, u64>(9)?,         // v.duration
            row.get::<_, u32>(10)?,        // t.id
            row.get::<_, u8>(11)?,         // t.title_index
            row.get::<_, u8>(12)?,         // t.media_type
            row.get::<_, String>(13)?,     // t.title
            row.get::<_, u16>(14)?,        // t.year
            row.get::<_, u16>(15)?,        // t.season
            row.get::<_, u16>(16)?,        // t.episode_number
            row.get::<_, u16>(17)?,        // t.episode_count
            row.get::<_, u8>(18)?,         // t.special_feature_kind
            row.get::<_, String>(19)?,     // t.special_feature_name
            row.get::<_, String>(20)?,     // t.version
            row.get::<_, u16>(21)?,        // t.disc
            row.get::<_, String>(22)?,     // t.location
            row.get::<_, String>(23)?,     // t.memo
        ))
    })?
    .map(|row| {
        let (
            id,
            location_path,
            checksum_hex,
            container_val,
            video_tracks_json,
            audio_tracks_json,
            subtitle_tracks_json,
            copy_op_id,
            transcode_op_id,
            duration_secs,
            title_id,
            title_index,
            media_type_val,
            title_name,
            year,
            season,
            episode_number,
            episode_count,
            sf_kind,
            sf_name,
            version,
            disc,
            title_location,
            memo,
        ) = row?;

        let checksum = blake3::Hash::from_hex(&checksum_hex)?;

        let container = conv::container_type_from_sql(container_val)?;

        let source = match (copy_op_id, transcode_op_id) {
            (Some(id), _) => VideoSource::CopyOperation(Reference { id, value: None }),
            (_, Some(id)) => VideoSource::TranscodeOperation(Reference { id, value: None }),
            _ => return Err(Error::InvalidVideoSource {
                copy_operation: copy_op_id,
                transcode_operation: transcode_op_id,
            }),
        };

        let title = Title {
            id: title_id,
            index: title_index,
            media_type: conv::media_type_from_sql(media_type_val)?,
            title: title_name,
            year,
            season,
            episode_number,
            episode_count,
            special_feature: conv::special_feature_from_sql(sf_kind, sf_name)?,
            version,
            disc,
            location: title_location,
            memo,
            videos: None,
        };

        Ok(Video {
            id,
            location: MediaLocation::Inbox(PathBuf::from(location_path)),
            checksum,
            container,
            video_tracks: serde_json::from_str(&video_tracks_json)?,
            audio_tracks: serde_json::from_str(&audio_tracks_json)?,
            subtitle_tracks: serde_json::from_str(&subtitle_tracks_json)?,
            source,
            title: Reference { id: title_id, value: Some(Box::new(title)) },
            duration: Duration::from_secs(duration_secs),
        })
    })
    .collect()
}

/// Updates the audio tracks field of a video record.
///
/// # Args
///
/// `conn`:  The connection to the database.
///
/// `id`:  The id of the video record to update.
///
/// `tracks`:  The audio tracks to store.
///
/// # Errors
///
/// [`crate::Error::Database`] raised if the database operation fails.
///
/// [`crate::Error::SerdeJson`] raised if the tracks cannot be serialized to JSON.
pub fn set_audio_tracks(conn: &Connection, id: u32, tracks: &[AudioTrack]) -> Result<()> {
    let sql = "UPDATE video SET audio_tracks = jsonb(?1) WHERE id = ?2";
    let json = serde_json::to_string(tracks)?;
    conn.execute(sql, (json, id))?;
    tracing::trace!(id, "set video audio_tracks");
    Ok(())
}

/// Updates the video tracks field of a video record.
///
/// # Args
///
/// `conn`:  The connection to the database.
///
/// `id`:  The id of the video record to update.
///
/// `tracks`:  The video tracks to store.
///
/// # Errors
///
/// [`crate::Error::Database`] raised if the database operation fails.
///
/// [`crate::Error::SerdeJson`] raised if the tracks cannot be serialized to JSON.
pub fn set_video_tracks(conn: &Connection, id: u32, tracks: &[VideoTrack]) -> Result<()> {
    let sql = "UPDATE video SET video_tracks = jsonb(?1) WHERE id = ?2";
    let json = serde_json::to_string(tracks)?;
    conn.execute(sql, (json, id))?;
    tracing::trace!(id, "set video video_tracks");
    Ok(())
}

/// Updates the subtitle tracks field of a video record.
///
/// # Args
///
/// `conn`:  The connection to the database.
///
/// `id`:  The id of the video record to update.
///
/// `tracks`:  The subtitle tracks to store.
///
/// # Errors
///
/// [`crate::Error::Database`] raised if the database operation fails.
///
/// [`crate::Error::SerdeJson`] raised if the tracks cannot be serialized to JSON.
pub fn set_subtitle_tracks(conn: &Connection, id: u32, tracks: &[SubtitleTrack]) -> Result<()> {
    let sql = "UPDATE video SET subtitle_tracks = jsonb(?1) WHERE id = ?2";
    let json = serde_json::to_string(tracks)?;
    conn.execute(sql, (json, id))?;
    tracing::trace!(id, "set video subtitle_tracks");
    Ok(())
}

/// Creates the database table for storing video data if it does not exist.
///
/// # Args
///
/// `conn`:  The connection to the database.
///
/// # Errors
///
/// [`crate::Error::Database`] raised if the database operation fails.
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

    let _ = conn.execute(sql, ())?;

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
        AudioCodec,
        AudioTrack,
        ContainerType,
        CopyOperation,
        MediaLocation,
        MediaType,
        Reference,
        SubtitleCodec,
        SubtitleTrack,
        VideoCodec,
        VideoSource,
        VideoTrack,
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

    #[test]
    fn test_inbox_videos_returns_inbox_videos() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video1 = make_video(copy_op_id, title_id);
        let mut video2 = make_video(copy_op_id, title_id);
        create(&conn, &mut video1).unwrap();
        create(&conn, &mut video2).unwrap();

        let inbox = inbox_videos(&conn).expect("Failed to list inbox");

        assert_eq!(inbox.len(), 2);
        assert!(inbox.iter().all(|v| matches!(v.location, MediaLocation::Inbox(_))));
    }

    #[test]
    fn test_inbox_videos_includes_title_data() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);
        create(&conn, &mut video).unwrap();

        let inbox = inbox_videos(&conn).expect("Failed to list inbox");

        assert_eq!(inbox.len(), 1);
        let title = inbox[0].title.value.as_ref().expect("title value should be populated");
        assert_eq!(title.id, title_id);
        assert_eq!(title.title, "Test Movie");
    }

    #[test]
    fn test_inbox_videos_excludes_library_videos() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut inbox_video = make_video(copy_op_id, title_id);
        let mut library_video = Video {
            location: MediaLocation::Library(std::path::PathBuf::from("movies/test.mkv")),
            ..make_video(copy_op_id, title_id)
        };
        create(&conn, &mut inbox_video).unwrap();
        create(&conn, &mut library_video).unwrap();

        let inbox = inbox_videos(&conn).expect("Failed to list inbox");

        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].id, inbox_video.id);
    }

    #[test]
    fn test_inbox_videos_empty() {
        let (conn, _, _) = setup_test_db();

        let inbox = inbox_videos(&conn).expect("Failed to list inbox");

        assert!(inbox.is_empty());
    }

    fn make_audio_track() -> AudioTrack {
        AudioTrack {
            container_index: 1,
            audio_index: 1,
            name: "English".to_owned(),
            codec: AudioCodec::AC3,
            encode_method: None,
            language_code: "eng".to_owned(),
            channel_count: 6,
            channel_layout: "5.1".to_owned(),
        }
    }

    fn make_video_track() -> VideoTrack {
        VideoTrack {
            container_index: 1,
            video_index: 1,
            codec: VideoCodec::H265,
            size: "1920x1080".to_owned(),
            aspect_ratio: "16:9".to_owned(),
        }
    }

    fn make_subtitle_track() -> SubtitleTrack {
        SubtitleTrack {
            container_index: 2,
            subtitle_index: 1,
            codec: SubtitleCodec::PGS,
            language_code: "eng".to_owned(),
        }
    }

    #[test]
    fn test_set_audio_tracks() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);
        create(&conn, &mut video).unwrap();
        let tracks = vec![make_audio_track()];

        set_audio_tracks(&conn, video.id, &tracks).expect("Failed to set audio tracks");

        let json: String = conn
            .query_row("SELECT json(audio_tracks) FROM video WHERE id = ?1", [video.id], |r| r.get(0))
            .unwrap();
        let stored: Vec<AudioTrack> = serde_json::from_str(&json).unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].language_code, "eng");
        assert_eq!(stored[0].channel_count, 6);
    }

    #[test]
    fn test_set_audio_tracks_empty() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);
        create(&conn, &mut video).unwrap();

        set_audio_tracks(&conn, video.id, &[]).expect("Failed to set empty audio tracks");

        let json: String = conn
            .query_row("SELECT json(audio_tracks) FROM video WHERE id = ?1", [video.id], |r| r.get(0))
            .unwrap();
        let stored: Vec<AudioTrack> = serde_json::from_str(&json).unwrap();
        assert!(stored.is_empty());
    }

    #[test]
    fn test_set_video_tracks() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);
        create(&conn, &mut video).unwrap();
        let tracks = vec![make_video_track()];

        set_video_tracks(&conn, video.id, &tracks).expect("Failed to set video tracks");

        let json: String = conn
            .query_row("SELECT json(video_tracks) FROM video WHERE id = ?1", [video.id], |r| r.get(0))
            .unwrap();
        let stored: Vec<VideoTrack> = serde_json::from_str(&json).unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].size, "1920x1080");
        assert_eq!(stored[0].aspect_ratio, "16:9");
    }

    #[test]
    fn test_set_video_tracks_empty() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);
        create(&conn, &mut video).unwrap();

        set_video_tracks(&conn, video.id, &[]).expect("Failed to set empty video tracks");

        let json: String = conn
            .query_row("SELECT json(video_tracks) FROM video WHERE id = ?1", [video.id], |r| r.get(0))
            .unwrap();
        let stored: Vec<VideoTrack> = serde_json::from_str(&json).unwrap();
        assert!(stored.is_empty());
    }

    #[test]
    fn test_set_subtitle_tracks() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);
        create(&conn, &mut video).unwrap();
        let tracks = vec![make_subtitle_track()];

        set_subtitle_tracks(&conn, video.id, &tracks).expect("Failed to set subtitle tracks");

        let json: String = conn
            .query_row("SELECT json(subtitle_tracks) FROM video WHERE id = ?1", [video.id], |r| r.get(0))
            .unwrap();
        let stored: Vec<SubtitleTrack> = serde_json::from_str(&json).unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].language_code, "eng");
    }

    #[test]
    fn test_set_subtitle_tracks_empty() {
        let (conn, copy_op_id, title_id) = setup_test_db();
        let mut video = make_video(copy_op_id, title_id);
        create(&conn, &mut video).unwrap();

        set_subtitle_tracks(&conn, video.id, &[]).expect("Failed to set empty subtitle tracks");

        let json: String = conn
            .query_row("SELECT json(subtitle_tracks) FROM video WHERE id = ?1", [video.id], |r| r.get(0))
            .unwrap();
        let stored: Vec<SubtitleTrack> = serde_json::from_str(&json).unwrap();
        assert!(stored.is_empty());
    }
}
