// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for defining the shared data models.

use std::fmt::{Display, Formatter, Result};
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use blake3::Hash;

use chrono::prelude::{DateTime, Utc};
use chrono::Datelike;

/// The range of valid years for movies and shows. 
///
/// Functionally, there is no reason the release year can be any valid number. This range limits
/// the value between the year when the first film was released and the current year since anything
/// outside this range is likely an input error by the user.
pub static RELEASE_YEAR_RANGE: LazyLock<RangeInclusive<u16>> = LazyLock::new(|| {
    // 1909 was the first release of a feature film. Also assume user isn't a time traveler.
    1909 ..= u16::try_from(Utc::now().year()).unwrap()
});

/// Specifies the various audio codecs.
///
/// This isn't meant to be an all inclusive list of audio codecs. It was generated using the 
/// HandBrake documentation.
#[derive(Debug)]
pub enum AudioCodec {
    /// Advanced Audio Coding (AAC)
    /// https://en.wikipedia.org/wiki/Advanced_Audio_Coding
    AAC,

    /// Dolby AC-3
    /// https://en.wikipedia.org/wiki/Dolby_Digital#Dolby_AC-3
    AC3,

    /// Apple Lossless Audio Codec (16 Channel)
    ALAC16,

    /// Apple Lossless Audio Codec (24 Channel)
    ALAC24,

    /// DTS Digital Surround
    /// https://en.wikipedia.org/wiki/DTS,_Inc.#DTS_Digital_Surround
    DTS,

    /// DTS-HD Master Audio
    /// https://en.wikipedia.org/wiki/DTS-HD_Master_Audio
    DTSHD,

    /// Dolby Digital Plus
    /// https://en.wikipedia.org/wiki/Dolby_Digital_Plus
    EAC3,

    /// Free Lossless Audio Codec (16 Channel)
    /// https://en.wikipedia.org/wiki/FLAC
    Flac16,

    /// Free Lossless Audio Codec (24 Channel)
    /// https://en.wikipedia.org/wiki/FLAC
    Flac24,

    /// MPEG-1 Audio Layer II
    /// https://en.wikipedia.org/wiki/MPEG-1_Audio_Layer_II
    MP2,

    /// MP3
    /// https://en.wikipedia.org/wiki/MP3
    MP3,

    /// Opus
    /// https://en.wikipedia.org/wiki/Opus_(audio_format)
    Opus,

    /// Dolby TrueHD
    /// https://en.wikipedia.org/wiki/Dolby_TrueHD
    TrueHD,

    /// Vorbis
    /// https://en.wikipedia.org/wiki/Vorbis
    Vorbis,
}

/// Specifies the methods of audio track encoding when transcoding.
#[derive(Debug)]
pub enum AudioEncodeMethod {
    /// Audio track is passed thru without modification.
    Copy,

    /// Audio track is re-encoded into [`AudioCodec::AAC`].
    AAC,
}

/// Media container types.
#[derive(Debug)]
pub enum ContainerType {
    /// Matroska Container Format
    /// https://en.wikipedia.org/wiki/Matroska
    MKV,

    /// MP4 (MPEG Part 14) Container Format 
    /// https://en.wikipedia.org/wiki/MP4_file_format
    MP4,
}

/// Location of a media file.
#[derive(Debug)]
pub enum MediaLocation {
    /// File path is relative to the media inbox root directory.
    Inbox(PathBuf),    

    /// File path is relative to the media library root directory.
    Library(PathBuf),

    /// File path is relative to the media archive root directory.
    Archive(PathBuf),

    /// File was deleted.
    Deleted,
}

/// Specifies the different types of media.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MediaType {
    #[default]
    Movie,
    Show,
}

impl MediaType {
    /// All available themes.
    pub const ALL: &'static [Self] = &[
        Self::Movie,
        Self::Show,
    ];
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            MediaType::Movie => write!(f, "Movie"),
            MediaType::Show => write!(f, "Show"),
        }
    }
}

/// Specifies the states of an operation (e.g. copy or transcode).
#[derive(Debug)]
pub enum OperationState {
    /// The operation was requested and waiting to be started.
    Requested,

    /// The operation is in-progress.
    Running,

    /// The operation completed successfully.
    Completed,

    /// The operation was cancelled.
    Cancelled,

    /// The operation failed to complete.
    Failed { reason: String },
}

/// Specifies the different types of movie and show extras.
#[derive(Debug)]
pub enum SpecialFeatureType {
    BehindTheScenes,
    DeletedScenes,
    Interviews,
    Scenes,
    Samples,
    Shorts,
    Featurettes,
    Clips,
    Extras,
    Trailers,
}

/// Specifies the various subtitle codecs.
#[derive(Debug)]
pub enum SubtitleCodec {
    /// EIA-608 Closed Caption Standard
    /// https://en.wikipedia.org/wiki/EIA-608
    CC,

    /// Presentation Graphic Stream
    /// https://en.wikipedia.org/wiki/Presentation_Graphic_Stream
    PGS,
}

/// Specifies the various video codecs.
///
/// This isn't meant to be an all inclusive list of video codecs. It was generated using the 
/// HandBrake documentation.
#[derive(Debug)]
pub enum VideoCodec {
    /// Advanced Video Coding (AVC) or H.264 or MPEG-4 Part 10
    /// https://en.wikipedia.org/wiki/Advanced_Video_Coding
    H264,

    /// High Efficiency Video Coding (HEVC) or H.265 or MPEG-H Part 2
    /// https://en.wikipedia.org/wiki/High_Efficiency_Video_Coding
    H265,

    /// MPEG-2
    /// https://en.wikipedia.org/wiki/MPEG-2
    MPEG2,

    /// MPEG-4
    /// https://en.wikipedia.org/wiki/MPEG-4
    MPEG4,

    /// Theora
    /// https://en.wikipedia.org/wiki/Theora
    Theora,

    /// VP8
    /// https://en.wikipedia.org/wiki/VP8
    VP8,

    /// VP9
    /// https://en.wikipedia.org/wiki/VP9
    VP9,
}

/// Specifies the different sources of a video file's creation.
#[derive(Debug)]
pub enum VideoSource {
    /// Video was created by copying a title from a DVD or Blu-ray.
    CopyOperation(Reference<CopyOperation>),

    /// Video was created by transcoding another video.
    TranscodeOperation(Reference<TranscodeOperation>),
}

/// Represents an audio track in a video.
#[derive(Debug)]
pub struct AudioTrack {
    /// The audio track's index.
    ///
    /// Audio track indexes will start at 1. So the first audio track at in the audio track vector
    /// of a video will have an audio track index of 1. This is to better match how external
    /// software typically numbers tracks.
    pub index: u8,

    /// The name of the audio track.
    ///
    /// The name is what will be displayed to users when selecting the audio track. That includes 
    /// both this application and external applications like VLC or Jellyfin. 
    pub name: String,

    /// The track's audio codec.
    pub codec: AudioCodec,

    /// Indicates how the audio track was encoded.
    ///
    /// `None` if the audio track is part of a video that was created as part of a copy operation
    /// or `Some` if that video was created by a transcoded operation.
    pub encode_method: Option<AudioEncodeMethod>,

    /// The audio track's language.
    pub language: String,

    /// The number of audio channels.
    ///
    /// Note that this is the total of all channels. So if the track is 5.1 surround, this value
    /// will be 6 (5 main channels + 1 subwoofer channel). 
    pub channel_count: u8,

    /// The channel layout.
    pub channel_layout: String,
}

/// Represents a copy operation.
///
/// There is intentionally some overlap between the fields in this data structure and the [`Title`]
/// data structure. The difference between the two is that this data structure won't change
/// once created whereas the fields in the [`Title`] might be edited.
#[derive(Debug)]
pub struct CopyOperation {
    /// Unique id of the copy operation (primary key).
    pub id: u32,

    /// Date/Time (UTC) when the operation was first requested.
    pub started: DateTime<Utc>,

    /// Date/Time (UTC) when the operation was completed, failed, or was cancelled.
    pub completed: DateTime<Utc>,

    /// The last known state of the operation.
    pub state: OperationState,

    /// The type of media the title is associated with.
    pub media_type: MediaType,

    /// The movie or show title.
    pub title: String,

    /// The release year.
    ///
    /// For television shows, this is the release year of the first season.
    pub year: u16,

    /// The disc number.
    pub disc: u16,

    /// The UUID of the disc as reported by the operating system.
    ///
    /// This may or may not be actually unique. It may also be possible that different values might
    /// be reported for the same disc on different operating systems or even between different
    /// drives. It is mainly being stored just in case it is needed for additional information.
    pub disc_uuid: String,

    /// The season number.
    ///
    /// Only valid for television shows. For movies, should be set to zero.
    pub season: u16,

    /// The physical location of the disc being copied.
    pub location: String,

    /// Additional information/context provided by the user.
    pub memo: String,

    /// The metadata extracted from the disc.
    ///
    /// This will be JSON data that maps to the `DiscInfo` data structure in the `makemkv` crate.
    pub metadata: String,

    /// The optical drive the copy operation was performed on.
    pub drive: Reference<OpticalDrive>,

    /// Raw log output captured when running the MakeMKV info command.
    pub info_log: String,

    /// Raw log output captured when running the MakeMKV copy command.
    pub copy_log: String,

    /// The computer the copy operation was performed on.
    pub host: Reference<Host>,

    /// List of titles created from this copy operation.
    ///
    /// Whether this is `Some` or `None` will depend on the database query.
    pub titles: Option<Vec<Title>>,

    /// List of videos created from this copy operation.
    ///
    /// Whether this is `Some` or `None` will depend on the database query.
    pub videos: Option<Vec<Video>>,
}

impl CopyOperation {
    // TODO
    pub fn from_params(params: &CopyParameters) -> Self {
        Self {
            media_type: params.media_type,
            title: params.title.clone(),
            year: params.release_year,
            season: params.season_number,
            disc: params.disc_number,
            location: params.location.clone(),
            memo: params.memo.clone(),
            ..CopyOperation::default()
        }
    }
}

impl Default for CopyOperation {
    fn default() -> Self {
        Self {
            id: 0,
            started: DateTime::<Utc>::default(),
            completed: DateTime::<Utc>::default(),
            state: OperationState::Requested,
            media_type: MediaType::Movie,
            title: String::default(),
            year: 0,
            disc: 0,
            disc_uuid: String::default(),
            season: 0,
            location: String::default(),
            memo: String::default(),
            metadata: String::default(),
            drive: Reference {
                id: 0,
                value: None
            },
            info_log: String::default(),
            copy_log: String::default(),
            host: Reference {
                id: 0,
                value: None
            },
            titles: None,
            videos: None,
        }
    }
}

/// The parameters for a copy operation.
#[derive(Clone, Debug)]
pub struct CopyParameters {
    /// The type of media being copied (Movie or TV Show).
    pub media_type: MediaType,

    /// The title of the show or movie.
    pub title: String,

    /// The release year of the movie or show (first season premier).
    pub release_year: u16,

    /// The season of the show the title belongs to.
    ///
    /// This is only required for television shows. It will be ignored for movies.
    pub season_number: u16,

    /// Disc number.
    pub disc_number: u16,

    /// Location where the disc is stored.
    pub location: String,

    /// Additional information provided by the user.
    pub memo: String,
}

impl CopyParameters {
    /// Returns true if the parameters are valid or false otherwise.
    pub fn valid(&self) -> bool {
        if self.title.trim().is_empty() {
            return false;
        }

        if self.location.trim().is_empty() {
            return false;
        }

        if !RELEASE_YEAR_RANGE.contains(&self.release_year) {
            return false;
        }

        if self.disc_number <= 0 {
            return false;
        }

        if self.media_type == MediaType::Show && self.season_number <= 0 {
            return false;
        }

        true
    }
}

/// Represents a specific computer an operation was performed on.
#[derive(Clone, Debug)]
pub struct Host {
    /// Unique id of the host (primary key).
    pub id: u32,

    /// The computer's hostname.
    pub hostname: String,
}

/// Represents an optical drive.
///
/// This is the representation of a drive within the database.
#[derive(Debug)]
pub struct OpticalDrive {
    /// Unique id of the drive (primary key).
    pub id: u32,

    /// Unique serial number assigned to the drive by the manufacturer. 
    pub serial_number: String,
}

/// Represents a relationship to another model.
#[derive(Debug)]
pub struct Reference<T> {
    /// The reference id (foreign key).
    pub id: u32,

    /// The reference data.
    ///
    /// Whether this is `Some` or `None` will depend on the database query. A boxed type is used
    /// here to avoid creating recursive types.
    pub value: Option<Box<T>>,
}

/// Represents a special feature in a DVD or Blu-ray.
#[derive(Debug)]
pub struct SpecialFeature {
    /// The type of special feature.
    pub kind: SpecialFeatureType,

    /// The name of the special feature.
    pub name: String,
}

/// Represents a subtitle track.
#[derive(Debug)]
pub struct SubtitleTrack {
    /// The subtitle's track index.
    ///
    /// Subtitle track indexes will start at 1. So the first audio track at in the subtitle track
    /// vector of a subtitle will have an subtitle track index of 1. This is to better match how
    /// external software typically numbers tracks.
    pub index: u8,
                                                                  
    /// The subtitle's codec.
    pub codec: SubtitleCodec,

    /// The subtitle language.
    pub language: String,
}

/// Represents an title.
///
/// A title can be a movie, TV show episode, or special feature. 
#[derive(Debug)]
pub struct Title {
    /// Unique id of the title (primary key).
    pub id: u32,

    /// The index of the title within the DVD or Blu-ray based how the titles were copied during
    /// the copy operation.
    pub index: u8,

    /// The type of media the title is associated with.
    pub media_type: MediaType,

    /// The movie or show title.
    pub title: String,

    /// The release year.
    ///
    /// For television shows, this is the release year of the first season.
    pub year: u16,

    /// The season number.
    ///
    /// Only valid for television shows. For movies, should be set to zero.
    pub season: u16,

    /// The episode number.
    ///
    /// In the case that this video covers multiple episodes, this will be the number for the first
    /// episode.
    ///
    /// This is only valid for shows. This will be set to zero for other media types. Additionally,
    /// this will not be set until the video is transcoded or catalogued. 
    pub episode_number: u16,

    /// The number of episodes this title covers.
    ///
    /// This is only valid for shows. This will be set to zero for other media types. Additionally,
    /// this will not be set until the video is transcoded or catalogued. 
    pub episode_count: u16,

    /// Special feature information.
    ///
    /// `None` if this video is not a special feature.
    pub special_feature: Option<SpecialFeature>,

    /// The version of the title (e.g. Directors Cut, 1080p, etc.)
    ///
    /// This should be empty for the default version of the title.
    pub version: String,

    /// The disc number the title was copied from.
    pub disc: u16,

    /// The physical location of the disc being copied.
    pub location: String,

    /// Additional information/context provided by the user.
    pub memo: String,

    /// List of videos associated with the title.
    ///
    /// Whether this is `Some` or `None` will depend on the database query.
    ///
    /// In general, each title will have one or two videos where one is from the copy operation and
    /// the other from the transcode operation.
    pub videos: Option<Vec<Title>>,
}

/// Represents a transcode operation.
///
/// There is intentionally some overlap between the fields in this data structure and the [`Title`]
/// data structure. The difference between the two is that this data structure won't change
/// once created whereas the fields in the [`Title`] might be edited.
#[derive(Debug)]
pub struct TranscodeOperation {
    /// Unique id of the operation (primary key).
    pub id: u32,

    /// Date/Time (UTC) when the operation was first requested.
    pub started: DateTime<Utc>,

    /// Date/Time (UTC) when the operation was completed, failed, or was cancelled.
    pub completed: DateTime<Utc>,

    /// The last known state of the operation.
    pub state: OperationState,

    /// The episode number.
    ///
    /// In the case that this video covers multiple episodes, this will be the number for the first
    /// episode.
    ///
    /// This is only valid for shows. This will be set to zero for other media types. Additionally,
    /// this will not be set until the video is transcoded or catalogued. 
    pub episode_number: u16,

    /// The number of episodes this title covers.
    ///
    /// This is only valid for shows. This will be set to zero for other media types. Additionally,
    /// this will not be set until the video is transcoded or catalogued. 
    pub episode_count: u16,

    /// Special feature information.
    ///
    /// `None` if this video is not a special feature.
    pub special_feature: Option<SpecialFeature>,

    /// The version of the title (e.g. Directors Cut, 1080p, etc.)
    ///
    /// This should be empty for the default version of the title.
    pub version: String,

    /// The audio track configuration.
    pub audio_tracks: Vec<AudioTrack>,

    /// The subtitle track configuration.
    pub subtitle_tracks: Vec<SubtitleTrack>,

    /// Raw log output captured when running the command.
    pub command_log: String,

    /// The computer the transcode operation was performed on.
    pub host: Reference<Host>,

    /// The title associated with the video being transcoded.
    pub title: Reference<Title>,

    /// The video being transcoded.
    pub source_video: Reference<Video>,
}

/// Represents an individual video file.
#[derive(Debug)]
pub struct Video {
    /// Unique id of the video (primary key).
    pub id: u32,

    /// The location of the video file.
    pub location: MediaLocation,

    /// The video file's checksum.
    pub checksum: Hash,

    /// The video container type (e.g. MKV or MP4).
    pub container: ContainerType,

    /// List of video tracks.
    ///
    /// This field is stored as JSON data in the database.
    pub video_tracks: Vec<VideoTrack>,

    /// List of audio tracks.
    ///
    /// This field is stored as JSON data in the database.
    pub audio_tracks: Vec<VideoTrack>,

    /// List of subtitle tracks.
    ///
    /// This field is stored as JSON data in the database.
    pub subtitle_tracks: Vec<SubtitleTrack>,

    /// The source of the video's creation.
    pub source: VideoSource,

    /// The associated title.
    pub title: Reference<Title>,

    /// The video's runtime.
    pub duration: Duration,
}

/// Represents a video track.
#[derive(Debug)]
pub struct VideoTrack {
    /// The video's track index.
    ///
    /// Video track indexes will start at 1. So the first audio track at in the video track vector
    /// of a video will have an video track index of 1. This is to better match how external
    /// software typically numbers tracks.
    pub index: u8,
                                                                  
    /// The video's codec.
    pub codec: VideoCodec,

    /// The video's size.
    pub size: String,

    /// The video's aspect ratio.
    pub aspect_ratio: String,
}

