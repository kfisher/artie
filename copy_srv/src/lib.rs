// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Crate that provides a service for copying titles from DVDs and Blu-rays.

use optical_drive::{self, OpticalDrive};

/// Result type for the `copy_srv` crate functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for `copy_srv` crate functions.
#[derive(Debug)]
pub enum Error {
    // Error emitted when the copy service cannot not acquire the exclusive lock for a drive. 
    DriveLocked,

    // Error emitted when the copy service cannot find or get information about an optical drive.
    InvalidOpticalDrive {
        error: Option<optical_drive::Error>,
    },
}

/// Service which handles copying titles from DVDs and Blu-rays discs.
///
/// Each service instance will handling copying from an individual optical drive.
#[derive(Clone)]
pub struct CopyService {
    /// The label assigned to the copy service instance.
    pub name: String,

    /// The drive this service instance is associated with.
    pub drive: OpticalDrive,
}

impl CopyService {
    /// Creates a new [`CopyService`] instance.
    ///
    /// # Errors
    ///
    /// This will return [`Error::InvalidOpticalDrive`] if an optical drive could not be found with
    /// the provided serial number either because it does not exist or because an error occurred
    /// when searching for the drive's information.
    ///
    /// [`Error::DriveLocked`] will be returned if another service instance already has the lock
    /// for the target drive.
    pub fn new(name: &str, serial_number: &str) -> Result<Self> {
        let drive = optical_drive::get_optical_drive(serial_number)
            .map_err(|error| Error::InvalidOpticalDrive { error: Some(error) })?
            .ok_or_else(|| Error::InvalidOpticalDrive { error: None })?;

        // TODO: Need to acquire an exclusive lock for the drive to prevent multiple instances
        //       performing operations for the same drive.

        Ok(Self {
            name: name.to_owned(),
            drive,
        })
    }
}

