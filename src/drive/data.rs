// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Persistent drive information.
//!
//! This module is used to save and load drive information like the current state and current form
//! data so that the information can persist across application runs.
//!
//! This does not include the drive's entry in the database which is handled by the database actor.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Persistent drive information.
#[derive(Deserialize, Debug, Default, Serialize)]
pub struct Data {
    /// The current form data.
    pub form: FormData,
}

impl Data {
    /// Loads the persistent data for a drive.
    ///
    /// If the file does not exist, the default values will be returned.
    ///
    /// # Errors
    ///
    /// [`Error::SerdeJson`] or [`Error::StdIo`] if an error occurs while trying to read or parse
    /// the data file if it exists.
    pub fn load(path: &Path) -> Result<Data> {
        if !path.exists() {
            return Err(Error::FileNotFound { path: path.to_owned() });
        }
        let text = fs::read_to_string(path)?;
        let data: Data = serde_json::from_str(&text)?;
        Ok(data)
    }

    /// Saves the persistent data for a drive.
    ///
    /// # Errors
    ///
    /// [`Error::SerdeJson`] or [`Error::StdIo`] if an error occurs while trying to serialize or
    /// write the data to the file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(path)?;
        file.write_all(text.as_bytes())?;
        Ok(())
    }
}

/// The persistent copy disc form information.
///
/// This allows the form values to be restored between application runs which is useful when
/// copying multiple discs with common data (e.g. multiple discs for a show).
#[derive(Deserialize, Debug, Default, Serialize)]
pub struct FormData {
    /// The type of media being copied (Movie or TV Show).
    pub media_type: String,

    /// The title of the show or movie.
    pub title: String,

    /// The release year of the movie or show (first season premier).
    pub year: String,

    /// Disc number.
    pub disc_number: String,

    /// The season of the show the title belongs to.
    ///
    /// This is only required for television shows. It will be ignored for movies.
    pub season_number: String,

    /// Location where the disc is stored.
    pub storage_location: String,

    /// Additional information provided by the user.
    pub memo: String,
}

/// Data used to update the form data in the drive's persisten data.
///
/// Each field is an option. If the value is `Some`, then the value was changed and needs updated.
/// If `None`, it should remain unchanged. Each field also has a corresponding factory method to
/// create an instance with its field filled in.
#[derive(Debug, Default)]
pub struct FormDataUpdate {
    /// The type of media being copied (Movie or TV Show).
    pub media_type: Option<String>,

    /// The title of the show or movie.
    pub title: Option<String>,

    /// The release year of the movie or show (first season premier).
    pub year: Option<String>,

    /// Disc number.
    pub disc_number: Option<String>,

    /// The season of the show the title belongs to.
    ///
    /// This is only required for television shows. It will be ignored for movies.
    pub season_number: Option<String>,

    /// Location where the disc is stored.
    pub storage_location: Option<String>,

    /// Additional information provided by the user.
    pub memo: Option<String>
}

impl FormDataUpdate {
    /// Create a form data update instance for updating the media type.
    pub fn media_type(value: String) -> Self {
        Self {
            media_type: Some(value),
            ..FormDataUpdate::default()
        }
    }

    /// Create a form data update instance for updating the title.
    pub fn title(value: String) -> Self {
        Self {
            title: Some(value),
            ..FormDataUpdate::default()
        }
    }

    /// Create a form data update instance for updating the yer.
    pub fn year(value: String) -> Self {
        Self {
            year: Some(value),
            ..FormDataUpdate::default()
        }
    }

    /// Create a form data update instance for updating the disc number.
    pub fn disc_number(value: String) -> Self {
        Self {
            disc_number: Some(value),
            ..FormDataUpdate::default()
        }
    }

    /// Create a form data update instance for updating the season number.
    pub fn season_number(value: String) -> Self {
        Self {
            season_number: Some(value),
            ..FormDataUpdate::default()
        }
    }

    /// Create a form data update instance for updating the storage location.
    pub fn storage_location(value: String) -> Self {
        Self {
            storage_location: Some(value),
            ..FormDataUpdate::default()
        }
    }

    /// Create a form data update instance for updating the memo.
    pub fn memo(value: String) -> Self {
        Self {
            memo: Some(value),
            ..FormDataUpdate::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::TempFile;

    #[test]
    fn test_save_and_load() {
        let temp_file = TempFile::new(Path::new("artie.test.drive.data.json"));

        let data = Data {
            form: FormData {
                media_type: String::from("Test Type"),
                title: String::from("Test Title"),
                year: String::from("Test Year"),
                disc_number: String::from("Test Disc Number"),
                season_number: String::from(""),
                storage_location: String::from("Test Location"),
                memo: String::from("Test Memo"),
            },
        };

        data.save(temp_file.path()).unwrap();

        let loaded_data = Data::load(temp_file.path()).unwrap();

        assert_eq!(data.form.media_type, loaded_data.form.media_type);
        assert_eq!(data.form.title, loaded_data.form.title);
        assert_eq!(data.form.year, loaded_data.form.year);
        assert_eq!(data.form.disc_number, loaded_data.form.disc_number);
        assert_eq!(data.form.season_number, loaded_data.form.season_number);
        assert_eq!(data.form.storage_location, loaded_data.form.storage_location);
        assert_eq!(data.form.memo, loaded_data.form.memo);
    }
}

