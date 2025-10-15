// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for defining the shared data models.

use std::fmt::{Display, Formatter, Result};

use chrono::prelude::{DateTime, Utc};

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

/// Represents a copy operation.
///
/// There is intentionally some overlap between the fields in this data structure and the [`Title`]
/// data structure. The difference between the two is that this data structure won't change
/// once created whereas the fields in the [`Title`] might be edited.
pub struct CopyOperation {
    /// Unique id of the copy operation (primary key).
    pub id: u32,

    /// Date/Time (UTC) when the operation was first requested.
    pub started: DateTime<Utc>,

    /// Date/Time (UTC) when the operation was completed, failed, or was cancelled.
    pub completed: DateTime<Utc>,

    /// The last known state of the operation.
    pub state: OperationState,

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

    /// The id of the optical drive the copy operation was performed on.
    pub drive_id: u32,

    /// The optical drive the copy operation was performed on.
    ///
    /// Whether this is `Some` or `None` will depend on the database query.
    pub drive: Option<OpticalDrive>,

    /// Raw log output captured when running the MakeMKV info command.
    pub info_log: String,

    /// Raw log output captured when running the MakeMKV copy command.
    pub copy_log: String,

    /// List of titles created from this copy operation.
    ///
    /// Whether this is `Some` or `None` will depend on the database query.
    pub titles: Option<Vec<Title>>,

    /// List of videos created from this copy operation.
    ///
    /// Whether this is `Some` or `None` will depend on the database query.
    pub videos: Option<Vec<Video>>,
}

/// Represents an optical drive.
///
/// This is the representation of a drive within the database.
pub struct OpticalDrive {
    /// Unique id of the drive (primary key).
    pub id: u32,

    /// Unique serial number assigned to the drive by the manufacturer. 
    pub serial_number: String,
}

// TODO: DOC
pub struct Title {
    /// Unique id of the title (primary key).
    pub id: u32,

    // TODO
}

/// Represents an individual video file.
pub struct Video {
    /// Unique id of the video (primary key).
    pub id: u32,

    // File Location
    // File Path 
    // Checksum
    // Video Tracks 
    // Audio Tracks 
    // Title Tracks

    // TODO
    // title 
    // copy operation
    // transcode operation
}

