// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Utilities to generate title and video records.
//!
//! The [`process_copy_operation`] function is used to process the title data extracted from a disc
//! which was copied. 

use rusqlite::{Connection, Transaction};

use makemkv::{DiscInfo, StreamInfo, TitleInfo};

use crate::{Error, Result};
use crate::db;
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
use crate::path;

/// Generates the title and video records in the database for each title that was copied during a
/// copy operation.
///
/// # Args
///
/// `copy_operation`:  The copy operation.
///
/// `serial_number`:  The serial number of the drive the copy operation was performed on.
///
/// `disc_info`:  The information that was extracted from the disc about the titles.
///
/// `conn`:  The database connection that should be used to generate the records.
///
/// # Errors
///
/// [`Error::Database`] if a database operation fails.
///
/// [`Error::MakeMkv`] if the required fields cannot be extracted from the title data either
/// because they are missing or malformed.
/// 
/// [`Error::MissingAudioCodecMapping`], [`Error::MissingSubtitleCodecMapping`], or
/// [`Error::MissingVideoCodecMapping`] if the codecs specified in the title data cannot be mapped
/// to one of the application codecs.
///
/// [`Error::UnexpectedFileExtension`] if a copy operation generated a video file with a video
/// extension other than `.mkv`. The check associated with this error is more of a sanity check
/// to verify an assumption then a true error.
pub fn process_copy_operation(
    copy_operation: &CopyOperation,
    serial_number: &str,
    disc_info: &DiscInfo,
    conn: &mut Connection,
) -> Result<()> {
    let transaction = db::transaction::start(conn)?;
  
    for (index, title_info) in disc_info.titles.iter().enumerate() {
        match title_info {
            Some(title_info) => {
                tracing::trace!(sn=serial_number, index, "processing title");
  
                // NOTE: The MakeMKV library has a hard limit on the number of titles of 100 titles
                //       which means index should always be a value u8.
                process_title_info(
                    copy_operation,
                    serial_number,
                    index as u8,
                    title_info,
                    &transaction,
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

/// Generate the title and video records in the database for a title generated from a copy
/// operation.
///
/// # Args
///
/// `copy_operation`:  The copy operation.
///
/// `serial_number`:  The serial number of the drive the copy operation was performed on.
///
/// `index`:  The title's index.
///
/// `title_info`:  The title information extracted from the disc during the copy operation.
///
/// `transaction`:  The database connection used to create the new records.  The changes to the
/// database will not be applied if there are any errors while processing all title data from the
/// copy operation.
///
/// # Errors
///
/// [`Error::Database`] if a database operation fails.
///
/// [`Error::MakeMkv`] if the required fields cannot be extracted from the title data either
/// because they are missing or malformed.
/// 
/// [`Error::MissingAudioCodecMapping`], [`Error::MissingSubtitleCodecMapping`], or
/// [`Error::MissingVideoCodecMapping`] if the codecs specified in the title data cannot be mapped
/// to one of the application codecs.
///
/// [`Error::UnexpectedFileExtension`] if a copy operation generated a video file with a video
/// extension other than `.mkv`. The check associated with this error is more of a sanity check
/// to verify an assumption then a true error.
fn process_title_info(
    copy_operation: &CopyOperation,
    serial_number: &str,
    index: u8,
    title_info: &TitleInfo,
    transaction: &Transaction,
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
  
    let file_name = title_info.output_file_name()?;
  
    if !file_name.ends_with(".mkv") {
        // The files created by the copy operation should always be an MKV file. This is a sanity
        // check to help ensure that remains the case.
        return Err(Error::UnexpectedFileExtension {
            expected: String::from("*.mkv"),
            actual: file_name.to_owned()
        });
    };
  
    let location = path::inbox_location(copy_operation, Some(&file_name));
  
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
  
    let duration = title_info.duration()?;
  
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

/// Process an audio track for a copied title and generate the audio data for it.
///
/// # Args
///
/// `index`:  The audio track's index. This is not the index of the stream within the title data,
/// but the index in relation to the other subtitle tracks. Additionally, the indexes start at 1.
///
/// `stream`:  The audio track data from the extracted disc information.
///
/// # Error
///
/// [`Error::MakeMkv`] if the required fields cannot be extracted from the stream data either
/// because they are missing or malformed.
///
/// [`Error::MissingAudioCodecMapping`] if the audio codec specified in the stream data is not
/// currently supported. This might just mean the MakeMKV identifier has not been mapped to the
/// application identifier for the codec.
fn process_audio_stream(index: usize, stream: &StreamInfo) -> Result<AudioTrack>{
    let codec = stream.codec()?;
  
    let track = AudioTrack {
        index: index as u8,
        name: stream.tree_info()?,
        codec: AudioCodec::from_makemkv(&codec)?,
        encode_method: None,
        language: stream.language_name()?,
        channel_count: stream.channel_count()?,
        channel_layout: stream.channel_layout()?,
    };
  
    Ok(track)
}

/// Process an subtitle track for a copied title and generate the subtitle track data for it.
///
/// # Args
///
/// `index`:  The subtitle track's index. This is not the index of the stream within the title
/// data, but the index in relation to the other subtitle tracks. Additionally, the indexes start
/// at 1.
///
/// `stream`:  The subtitle track data from the extracted disc information.
///
/// # Error
///
/// [`Error::MakeMkv`] if the required fields cannot be extracted from the stream data either
/// because they are missing or malformed.
///
/// [`Error::MissingSubtitleCodecMapping`] if the subtitle codec specified in the stream data is
/// not currently supported. This might just mean the MakeMKV identifier has not been mapped to the
/// application identifier for the codec.
fn process_subtitle_stream(index: usize, stream: &StreamInfo) -> Result<SubtitleTrack> {
    let codec = stream.codec()?;
  
    let track = SubtitleTrack {
        index: index as u8,
        codec: SubtitleCodec::from_makemkv(&codec)?,
        language: stream.language_name()?,
    };
  
    Ok(track)
}

/// Process an video track for a copied title and create the video track data for it.
///
/// # Args
///
/// `index`:  The video track's index. This is not the index of the stream within the title data,
/// but the index in relation to the other video tracks. Additionally, the indexes start at 1.
///
/// `stream`:  The video track data from the extracted disc information.
///
/// # Error
///
/// [`Error::MakeMkv`] if the required fields cannot be extracted from the stream data either
/// because they are missing or malformed.
///
/// [`Error::MissingVideoCodecMapping`] if the video codec specified in the stream data is not
/// currently supported. This might just mean the MakeMKV identifier has not been mapped to the
/// application identifier for the codec.
fn process_video_stream(index: usize, stream: &StreamInfo) -> Result<VideoTrack> {
    let codec = stream.codec()?;
  
    let track = VideoTrack {
        index: index as u8,
        codec: VideoCodec::from_makemkv(&codec)?,
        size: stream.video_size()?,
        aspect_ratio: stream.aspect_radio()?,
    };
  
    Ok(track)
}

#[cfg(test)]
mod tests {
    // TODO
}
