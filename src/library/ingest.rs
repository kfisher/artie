// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only


use rusqlite::{Connection, Transaction};

use makemkv::{DiscInfo, StreamInfo, TitleInfo};

use crate::{Error, Result};
use crate::db::{self};
use crate::fs::FileSystem;
use crate::models::{
    AudioCodec,
    AudioTrack,
    ContainerType,
    CopyOperation,
    Reference,
    SubtitleCodec,
    SubtitleTrack,
    Title,
    Video,
    VideoCodec,
    VideoSource,
    VideoTrack,
};

/// Takes a copy operation and generates the [`Title`] and [`Video`] entries in the database for
/// each title that was copied.
pub fn process_copy_operation(
    fs: &FileSystem,
    serial_number: &str,
    conn: &mut Connection,
    disc_info: &DiscInfo,
    copy_operation: &CopyOperation,
) -> Result<()> {
    let transaction = db::transaction::start(conn)?;

    for (index, title_info) in disc_info.titles.iter().enumerate() {
        match title_info {
            Some(title_info) => {
                tracing::trace!(sn=serial_number, index, "processing title");

                // NOTE: The MakeMKV library has a hard limit on the number of titles of 100 titles
                //       which means index should always be a value u8.
                process_title_info(
                    fs,
                    serial_number,
                    &transaction,
                    index as u8,
                    copy_operation,
                    title_info
                )?;
            },
            None => {
                tracing::warn!(sn=serial_number, index, "missing title information");
            }
        }
    }

    db::transaction::commit(transaction)?;
    Ok(())
}

/// Generate the [`Title`] and [`Video`] entries in the database for a title generated from a copy
/// operation.
fn process_title_info(
    fs: &FileSystem,
    serial_number: &str,
    transaction: &Transaction,
    index: u8,
    copy_operation: &CopyOperation,
    title_info: &TitleInfo,
) -> Result<()> {
    let mut title = Title {
        id: 0,
        index,
        media_type: copy_operation.media_type,
        title: copy_operation.title.clone(),
        year: copy_operation.year,
        season: copy_operation.season,
        episode_number: 0,
        episode_count: 0,
        special_feature: None,
        version: String::default(),
        disc: copy_operation.disc,
        location: copy_operation.location.clone(),
        memo: copy_operation.memo.clone(),
        videos: None,
    };

    db::title::create(transaction, &mut title)?;

    let file_name = title_info.output_file_name()
        .map_err(|error| Error::MakeMKV { error })?;

    if !file_name.ends_with(".mkv") {
        // The files created by the copy operation should always be an MKV file. This is a sanity
        // check to help ensure that remains the case. 
        return Err(Error::UnexpectedFileExtension {
            expected: String::from("*.mkv"),
            actual: file_name.to_owned() 
        });
    };

    let location = fs.inbox_location(copy_operation, Some(&file_name));

    // TODO
    let checksum: [u8; 32] = [0; 32];
    let checksum = blake3::Hash::from_bytes(checksum);

    let container = ContainerType::MKV;

    let mut video_tracks: Vec<VideoTrack> = vec![];
    let mut audio_tracks: Vec<AudioTrack> = vec![];
    let mut subtitle_tracks: Vec<SubtitleTrack> = vec![];

    for stream in title_info.streams.iter() {
        let Some(stream) = stream else {
            tracing::warn!(sn=serial_number, index, "missing stream information");
            continue;
        };

        if stream.is_video_stream() {
            video_tracks.push(process_video_stream(video_tracks.len() + 1, stream)?);
        } else if stream.is_audio_stream() {
            audio_tracks.push(process_audio_stream(audio_tracks.len() + 1, stream)?);
        } else if stream.is_subtitle_stream() {
            subtitle_tracks.push(process_subtitle_stream(subtitle_tracks.len() + 1, stream)?);
        } else {
            return Err(Error::UnexpectedStreamType { 
                stream_type: stream.stream_type().ok(),
            });
        }
    }

    let source = VideoSource::CopyOperation(Reference { id: copy_operation.id, value: None });

    let duration = title_info.duration()
        .map_err(|error| Error::MakeMKV { error })?;

    let mut video = Video {
        id: 0,
        location,
        checksum,
        container,
        video_tracks,
        audio_tracks,
        subtitle_tracks,
        source,
        title: Reference { id: title.id, value: None },
        duration,
    };

    db::video::create(transaction, &mut video)?;

    Ok(())
}

/// Process an audio track for a copied title and generate an [`AudioTrack`] for it.
fn process_audio_stream(index: usize, stream: &StreamInfo) -> Result<AudioTrack>{
    let codec = stream.codec()
        .map_err(|error| Error::MakeMKV { error })?;

    let track = AudioTrack {
        index: index as u8,
        name: stream.tree_info()
            .map_err(|error| Error::MakeMKV { error })?,
        codec: AudioCodec::from_makemkv(&codec)?,
        encode_method: None,
        language: stream.language_name()
            .map_err(|error| Error::MakeMKV { error })?,
        channel_count: stream.channel_count()
            .map_err(|error| Error::MakeMKV { error })?,
        channel_layout: stream.channel_layout()
            .map_err(|error| Error::MakeMKV { error })?,
    };

    Ok(track)
}

/// Process an subtitle track for a copied title and generate an [`SubtitleTrack`] for it.
fn process_subtitle_stream(index: usize, stream: &StreamInfo) -> Result<SubtitleTrack> {
    let codec = stream.codec()
        .map_err(|error| Error::MakeMKV { error })?;

    let track = SubtitleTrack {
        index: index as u8,
        codec: SubtitleCodec::from_makemkv(&codec)?,
        language: stream.language_name()
            .map_err(|error| Error::MakeMKV { error })?,
    };

    Ok(track)
}

/// Process an video track for a copied title and generate an [`VideoTrack`] for it.
fn process_video_stream(index: usize, stream: &StreamInfo) -> Result<VideoTrack> {
    let codec = stream.codec()
        .map_err(|error| Error::MakeMKV { error })?;

    let track = VideoTrack {
        index: index as u8,
        codec: VideoCodec::from_makemkv(&codec)?,
        size: stream.video_size()
            .map_err(|error| Error::MakeMKV { error })?,
        aspect_ratio: stream.aspect_radio()
            .map_err(|error| Error::MakeMKV { error })?,
    };

    Ok(track)
}

// TODO: Testing
