// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate that provides a service for copying titles from DVDs and Blu-rays.

use std::time::Duration;

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

    /// Error emitted when the copy service fails to complete an action because a copy operation
    /// is currently in progress.
    OperationInProgress,
}

/// Specifies the states of the copy service.
#[derive(Debug, PartialEq)]
pub enum State {
    /// The copy service cannot get the drive information.
    ///
    /// This can happen because the drive is disconnected or the service cannot communicate with
    /// the remote node.
    Disconnected,

    /// The copy service is idle.
    Idle,

    /// The copy service is in the process of copying media.
    Copying {
        stage: String,
        task: String,
        subtask: String,
        task_progress: f32,
        subtask_progress: f32,
        elapsed_time: Duration,
    },

    /// The copy operation completed successfully.
    ///
    /// Once the copy operation completes, the service will remain in this state until the user
    /// acknowledges it which will reset the state back to `Idle`.
    Success,

    /// The copy operation failed or was cancelled.
    ///
    /// The service will remain in this state until the user acknowledges the error which will
    /// reset the state back to `Idle`.
    Failed {
        error: String,
    },
}

impl State {
    /// Returns `true` if a copy operation is in-progress.
    ///
    /// A copy operation is considered in-progress if copying and if it is in the failed or success
    /// end states.
    pub fn operation_in_progress(&self) -> bool {
        matches!(self, State::Copying {..} | State::Failed {..} | State::Success)
    }
}

/// Service which handles copying titles from DVDs and Blu-rays discs.
///
/// Each service instance will handling copying from an individual optical drive.
#[derive(Debug, PartialEq)]
pub struct CopyService {
    /// The label assigned to the copy service instance.
    name: String,

    /// The serial number of the drive.
    serial_number: String,

    /// The drive this service instance is associated with.
    drive: Option<OpticalDrive>,

    /// The current state of the service.
    state: State,
}

impl CopyService {
    // TODO: Need to acquire an exclusive lock for the drive to prevent multiple instances
    //       performing operations for the same drive. Will need to ensure the lock is 
    //       correctly released when: service destroyed, drive changed.

    /// Gets the name assigned to the service.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the serial number for the optical drive associated with the service.
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Gets the optical drive data for the drive associated with the service.
    ///
    /// `Some` if the service was able to get the data from the OS or `None` if the OS was not able
    /// to find a drive with the serial number the service was configured with.
    pub fn drive(&self) -> Option<&OpticalDrive> {
        self.drive.as_ref()
    }

    /// Gets the state of the service.
    pub fn state(&self) -> &State {
        &self.state
    }

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
        let mut service = Self {
            name: name.to_owned(),
            serial_number: serial_number.to_owned(),
            drive: None,
            state: State::Disconnected,
        };

        service.refresh_drive()?;

        Ok(service)
    }

    /// Updates the copy service configuration.
    ///
    /// This will update the optical drive information and state accordingly if the serial number
    /// changed. Therefore, this cannot be called when a copy operation is currently in-progress
    /// which includes the failed and success end states.
    ///
    /// # Errors
    ///
    /// - [`Error::DriveLocked`] if the serial number was changed and another service instance 
    ///   already has the lock for that drive.
    /// - [`Error::InvalidOpticalDrive`] if the serial number was changed and the request to get
    ///   the optical drive failed.
    /// - [`Error::OperationInProgress`] if called while a copy operation is in progress. This
    ///   includes the success and failed end states.
    pub fn update_config(&mut self, name: &str, serial_number: &str) -> Result<()> {
        if self.state.operation_in_progress() {
            return Err(Error::OperationInProgress);
        }

        self.name = name.to_owned();

        if self.serial_number != serial_number {
            self.serial_number = serial_number.to_owned();
            self.refresh_drive()?;
        }

        Ok(())
    }

    /// Updates the optical drive data.
    ///
    /// This will get the optical drive information from the OS and then update the state of the 
    /// service accodingly. Therefore, this cannot be called when a copy operation is currently
    /// in-progress (includes end states).
    ///
    /// # Errors
    ///
    /// - [`Error::DriveLocked`] if another service instance already has the lock.
    /// - [`Error::InvalidOpticalDrive`] if the request to get the optical drive failed.
    /// - [`Error::OperationInProgress`] if called while a copy operation is in progress. This
    ///   includes the success and failed end states.
    fn refresh_drive(&mut self) -> Result<()> {
        if self.state.operation_in_progress() {
            return Err(Error::OperationInProgress);
        }

        self.drive = optical_drive::get_optical_drive(&self.serial_number)
            .map_err(|e| Error::InvalidOpticalDrive { error: Some(e) })?;

        self.state = match self.drive {
            Some(_) => State::Idle,
            None => State::Disconnected,
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use optical_drive::{DiscState, OpticalDrive};

    // NOTE: The following depends on the 'faux_drives' feature being enabled for the optical_drive
    //       dependency since it uses the mock drives defined within that feature.

    // NOTE: The name field is allowed to be empty within the context of the service. It is up to 
    //       the UI to enforce it not being empty or handle cases when it is.

    fn faux_drive_1() -> OpticalDrive {
        OpticalDrive {
            path: String::from("/dev/fx0"),
            serial_number: String::from("FAUX0001"),
            disc: DiscState::None,
        }
    }

    fn faux_drive_2() -> OpticalDrive {
        OpticalDrive {
            path: String::from("/dev/fx1"),
            serial_number: String::from("FAUX0002"),
            disc: DiscState::Inserted {
                label: String::from("FAUX_MOVIE"),
                uuid: String::from("00000000-0000-0000-0000-000000000000"),
            },
        }
    }

    #[test]
    fn copy_service_new() {
        // FAUX0000 doesn't exist, so the service should initialize
        let service = CopyService::new("TestDrive", "FAUX0000")
            .expect("Unexpected error when creating copy service");

        assert_eq!(service.name(), "TestDrive");
        assert_eq!(service.serial_number(), "FAUX0000");
        assert_eq!(service.drive(), None);
        assert_eq!(service.state(), &State::Disconnected);

        // FAUX0001 is an empty drive.
        let service = CopyService::new("TestDrive", "FAUX0001")
            .expect("Unexpected error when creating copy service");

        assert_eq!(service.name(), "TestDrive");
        assert_eq!(service.serial_number(), "FAUX0001");
        assert_eq!(service.drive(), Some(&faux_drive_1()));
        assert_eq!(service.state(), &State::Idle);

        // FAUX0002 has a disc inserted.
        let service = CopyService::new("TestDrive", "FAUX0002")
            .expect("Unexpected error when creating copy service");

        assert_eq!(service.name(), "TestDrive");
        assert_eq!(service.serial_number(), "FAUX0002");
        assert_eq!(service.drive(), Some(&faux_drive_2()));
        assert_eq!(service.state(), &State::Idle);
    }

    #[test]
    fn copy_service_update() {
        let mut service = CopyService::new("TestDrive 0", "FAUX0000")
            .expect("Unexpected error when creating copy service");

        service.update_config("TestDrive 1", "FAUX0001")
            .expect("Unexpected error when updating copy service");
        assert_eq!(service.name(), "TestDrive 1");
        assert_eq!(service.serial_number(), "FAUX0001");
        assert_eq!(service.drive(), Some(&faux_drive_1()));
        assert_eq!(service.state(), &State::Idle);

        service.update_config("TestDrive 2", "FAUX0002")
            .expect("Unexpected error when updating copy service");
        assert_eq!(service.name(), "TestDrive 2");
        assert_eq!(service.serial_number(), "FAUX0002");
        assert_eq!(service.drive(), Some(&faux_drive_2()));
        assert_eq!(service.state(), &State::Idle);

        service.update_config("TestDrive 3", "FAUX0000")
            .expect("Unexpected error when updating copy service");
        assert_eq!(service.name(), "TestDrive 3");
        assert_eq!(service.serial_number(), "FAUX0000");
        assert_eq!(service.drive(), None);
        assert_eq!(service.state(), &State::Disconnected);
    }

    // TODO: Need to verify that attempting to update the service while a copy operation is in 
    //       progress correctly fails.
}

