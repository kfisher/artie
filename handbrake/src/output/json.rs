// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the JSON data structures for Handbrake output.

use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// The progress data when the state is "MUXING".
#[derive(Serialize, Deserialize)]
pub struct Muxing {
    #[serde(rename = "Progress")]
    pub progress: f32,
}

/// The progress data from a running Handbrake command.
#[derive(Serialize, Deserialize)]
pub struct Progress {
    #[serde(rename = "State")]
    pub state: String,

    #[serde(rename = "Muxing")]
    pub muxing: Option<Muxing>,

    #[serde(rename = "Working")]
    pub working: Option<Working>,

    #[serde(rename = "WorkDone")]
    pub work_done: Option<WorkDone>,
}

/// Represents Handbrake's version number.
#[derive(Serialize, Deserialize)]
pub struct SemanticVersion {
    #[serde(rename = "Major")]
    pub major: i32,

    #[serde(rename = "Minor")]
    pub minor: i32,

    #[serde(rename = "Point")]
    pub point: i32,
}

/// The version data from a running Handbrake command.
#[derive(Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "Arch")]
    pub arch: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Official")]
    pub official: bool,

    #[serde(rename = "RepoDate")]
    pub repo_date: String,

    #[serde(rename = "RepoHash")]
    pub repo_hash: String,

    #[serde(rename = "System")]
    pub system: String,

    #[serde(rename = "Type")]
    pub release_type: String,

    #[serde(rename = "Version")]
    pub version: SemanticVersion,

    #[serde(rename = "VersionString")]
    pub version_string: String,
}

/// The progress data when the state is "WORKING".
#[derive(Serialize, Deserialize)]
pub struct Working {
    #[serde(rename = "ETASeconds")]
    pub eta_seconds: i32,

    #[serde(rename = "Hours")]
    pub hours: i32,

    #[serde(rename = "Minutes")]
    pub minutes: i32,

    #[serde(rename = "Pass")]
    pub pass: i32,

    #[serde(rename = "PassCount")]
    pub pass_count: i32,

    #[serde(rename = "PassID")]
    pub pass_id: i32,

    #[serde(rename = "Paused")]
    pub paused: i32,

    #[serde(rename = "Progress")]
    pub progress: f32,

    #[serde(rename = "Rate")]
    pub rate: f32,

    #[serde(rename = "RateAvg")]
    pub rate_avg: f32,

    #[serde(rename = "Seconds")]
    pub seconds: i32,

    #[serde(rename = "SequenceID")]
    pub sequence_id: i32,
}

/// The progress data when the state is "WORKDONE".
#[derive(Serialize, Deserialize)]
pub struct WorkDone {
    #[serde(rename = "Error")]
    pub error: i32,

    #[serde(rename = "SequenceID")]
    pub sequence_id: i32,
}

/// Parses a progress JSON object from the provided reader.
pub fn parse_progress<T>(reader: &mut T) -> Result<Progress> 
where
    T: Read
{
    serde_json::from_reader(reader).map_err(|e| Error::JsonParseError { error: e })
}

/// Parses version information from Handbrake's output.
pub fn parse_version<T>(reader: &mut T) -> Result<Version> 
where
    T: Read
{
    serde_json::from_reader(reader).map_err(|e| Error::JsonParseError { error: e })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    #[test]
    fn parse_progress_muxing() {
        let bytes = br#"{
    "Muxing": {
        "Progress": 0.0
    },
    "State": "MUXING"
}
"#;

        let mut buffer = Cursor::new(bytes);
        let progress = super::parse_progress(&mut buffer).unwrap();

        assert_eq!(progress.state, "MUXING");
        assert_eq!(progress.muxing.is_some(), true);
        assert_eq!(progress.working.is_some(), false);
        assert_eq!(progress.work_done.is_some(), false);

        let muxing = progress.muxing.unwrap();
        assert_eq!(muxing.progress, 0.);
    }

    #[test]
    fn parse_progress_working() {
        let bytes = br#"{
    "State": "WORKING",
    "Working": {
        "ETASeconds": 1,
        "Hours": 2,
        "Minutes": 3,
        "Pass": 1,
        "PassCount": 2,
        "PassID": -1,
        "Paused": 0,
        "Progress": 0.095,
        "Rate": 0.0,
        "RateAvg": 0.0,
        "Seconds": 1,
        "SequenceID": 1
    }
}
"#;
        let mut buffer = Cursor::new(bytes);
        let progress = super::parse_progress(&mut buffer).unwrap();

        assert_eq!(progress.state, "WORKING");
        assert_eq!(progress.muxing.is_some(), false);
        assert_eq!(progress.working.is_some(), true);
        assert_eq!(progress.work_done.is_some(), false);

        let working = progress.working.unwrap();
        assert_eq!(working.eta_seconds, 1);
        assert_eq!(working.hours, 2);
        assert_eq!(working.minutes, 3);
        assert_eq!(working.pass, 1);
        assert_eq!(working.pass_count, 2);
        assert_eq!(working.pass_id, -1);
        assert_eq!(working.paused, 0);
        assert_eq!(working.progress, 0.095);
        assert_eq!(working.rate, 0.0);
        assert_eq!(working.rate_avg, 0.0);
        assert_eq!(working.seconds, 1);
        assert_eq!(working.sequence_id, 1);
    }

    #[test]
    fn parse_progress_workdone() {
        let bytes = br#"{
    "State": "WORKDONE",
    "WorkDone": {
        "Error": 0,
        "SequenceID": 1
    }
}
"#;

        let mut buffer = Cursor::new(bytes);
        let progress = super::parse_progress(&mut buffer).unwrap();

        assert_eq!(progress.state, "WORKDONE");
        assert_eq!(progress.muxing.is_some(), false);
        assert_eq!(progress.working.is_some(), false);
        assert_eq!(progress.work_done.is_some(), true);

        let work_done = progress.work_done.unwrap();
        assert_eq!(work_done.error, 0);
        assert_eq!(work_done.sequence_id, 1);
    }

    #[test]
    fn parse_progress_invalid_json() {
        let bytes = br#"{
    "WorkDone": {
        "Error": 0,
        "SequenceID": 1
    }
}
"#;

        let mut buffer = Cursor::new(bytes);
        if super::parse_progress(&mut buffer).is_ok() {
            panic!("Expected an error")
        }
    }

    #[test]
    fn parse_version() {
        let bytes = br#"{
    "Arch": "x86_64",
    "Name": "HandBrake",
    "Official": true,
    "RepoDate": "2024-08-07 17:31:52",
    "RepoHash": "77f199ab02ff2e3bca4ca653e922e9fef67dec43",
    "System": "MinGW",
    "Type": "release",
    "Version": {
        "Major": 1,
        "Minor": 8,
        "Point": 2
    },
    "VersionString": "1.8.2"
}
"#;

        let mut buffer = Cursor::new(bytes);
        let version = super::parse_version(&mut buffer).unwrap();

        assert_eq!(version.arch, "x86_64");
        assert_eq!(version.name, "HandBrake");
        assert_eq!(version.official, true);
        assert_eq!(version.repo_date, "2024-08-07 17:31:52");
        assert_eq!(version.repo_hash, "77f199ab02ff2e3bca4ca653e922e9fef67dec43");
        assert_eq!(version.system, "MinGW");
        assert_eq!(version.release_type, "release");
        assert_eq!(version.version.major, 1);
        assert_eq!(version.version.minor, 8);
        assert_eq!(version.version.point, 2);
        assert_eq!(version.version_string, "1.8.2");
    }

    #[test]
    fn parse_version_invalid_json() {
        let bytes = br#"{
    "Arch": "x86_64",
    "Version": {
        "Major": 1,
        "Minor": 8,
        "Point": 2
    },
    "VersionString": "1.8.2"
}
"#;
        let mut buffer = Cursor::new(bytes);
        if super::parse_version(&mut buffer).is_ok() {
            panic!("Expected an error")
        }
    }
}
