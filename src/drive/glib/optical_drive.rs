// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the GObject representing an optical drive.

use std::time::Duration;

use gtk::glib;
use gtk::glib::Object;
use gtk::subclass::prelude::*;

use crate::drive::OpticalDrive;
use crate::drive::actor;
use crate::drive::data::{FormData, FormDataUpdate};
use crate::drive::glib::OpticalDriveState;
use crate::db::Database;
use crate::fs::FileSystem;
use crate::models::CopyParamaters;

glib::wrapper! {
    /// GObject implementation for [`OpticalDrive`].
    pub struct OpticalDriveObject(ObjectSubclass<imp::OpticalDriveObject>);
}

impl OpticalDriveObject {
    /// Creates new [`OpticalDriveObject`] instance from the provided optical drive.
    pub fn new(drive: OpticalDrive, fs: FileSystem, db: Database) -> Self {
        let obj: Self = Object::builder()
            .build();

        let imp = obj.imp();

        imp.handle.replace(Some(actor::local::create_actor(&drive, fs, db)));

        imp.name.replace(drive.serial_number.clone());
        imp.serial_number.replace(drive.serial_number);
        imp.path.replace(drive.path);

        obj
    }

    /// Cancels a copy operation currently in progress.
    pub async fn cancel_copy_disc(&self) {
        let handle = self.imp().handle
            .borrow()
            .as_ref()
            .expect("FIXME: error msg or proper handling")
            .clone();
        let result = handle
            .cancel_copy_disc()
            .await;
        if let Err(error) = result {
            tracing::error!(?error, "failed to cancel copy disc");
        }
    }

    /// Begin copying the disc in the optical drive.
    pub async fn copy_disc(&self, copy_parameters: CopyParamaters) {
        let handle = self.imp().handle
            .borrow()
            .as_ref()
            .expect("FIXME: error msg or proper handling")
            .clone();
        let result = handle
            .copy_disc(copy_parameters)
            .await;
        if let Err(error) = result {
            tracing::error!(?error, "failed to copy disc");
        }
    }

    /// Get the form data from the drive's persistent storage.
    pub async fn get_form_data(&self) -> Option<FormData> {
        let handle = self.imp().handle
            .borrow()
            .as_ref()
            .expect("FIXME: error msg or proper handling")
            .clone();
        let result = handle
            .get_form_data()
            .await;
        match result {
            Ok(form_data) => Some(form_data),
            Err(error) => {
                tracing::error!(?error, "failed to get form data");
                None
            }
        }
    }

    /// Reset the drive state back to `Idle` from `Success` or `Failed`.
    pub async fn reset(&self) {
        let handle = self.imp().handle
            .borrow()
            .as_ref()
            .expect("FIXME: error msg or proper handling")
            .clone();
        let result = handle
            .reset()
            .await;
        if let Err(error) = result {
            tracing::error!(?error, "failed to reset");
        }
    }

    /// Updates the form data in the drive's persistent data.
    pub async fn update_form_data(&self, data: FormDataUpdate) {
        let handle = self.imp().handle
            .borrow()
            .as_ref()
            .expect("FIXME: error msg or proper handling")
            .clone();
        let result = handle
            .update_form_data(data)
            .await;

        if let Err(error) = result {
            tracing::error!(?error, "failed to update form data");
        }
    }

    /// Updates the status of the drive.
    ///
    /// This will request the current status from the drive's actor instance and then send the
    /// property change notifications if required.
    pub async fn update_status(&self) {
        let handle = self.imp().handle
            .borrow()
            .as_ref()
            .expect("FIXME: error msg or proper handling")
            .clone();
        let status = handle
            .get_status()
            .await;
        let status = match status {
            Ok(status) => status,
            Err(error) => {
                tracing::error!(?error, "failed to update status");
                return;
            }
        };

        match status.disc {
            crate::drive::DiscState::None => {
                self.set_disc_label(String::default());
            },
            crate::drive::DiscState::Inserted { label, uuid: _ } => {
                self.set_disc_label(label);
            },
        }

        match status.drive {
            crate::drive::OpticalDriveState::Disconnected => {
                self.set_drive_state(OpticalDriveState::Disconnected);
            },
            crate::drive::OpticalDriveState::Idle => {
                self.set_drive_state(OpticalDriveState::Idle);
            },
            crate::drive::OpticalDriveState::Copying {
                stage,
                task,
                task_progress,
                subtask,
                subtask_progress,
                elapsed_time,
            } => {
                self.set_drive_state(OpticalDriveState::Copying);
                self.set_stage(stage);
                self.set_elapsed_time(format_elapsed_time(&elapsed_time));
                self.set_task(task);
                self.set_task_progress(task_progress);
                self.set_subtask(subtask);
                self.set_subtask_progress(subtask_progress);
            },
            crate::drive::OpticalDriveState::Success => {
                self.set_drive_state(OpticalDriveState::Success);
            },
            crate::drive::OpticalDriveState::Failed { error } => {
                self.set_drive_state(OpticalDriveState::Failed);
                self.set_error_message(error);
            },
        }
    }

}

/// Formats the elapsed time duration into a string.
fn format_elapsed_time(elapsed_time: &Duration) -> String {
    let total_seconds = elapsed_time.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

mod imp {
    //! Implementation for the optical drive object.

    use std::cell::{Cell, RefCell};
    

    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    
    use crate::drive::actor::handle::DriveActorHandle;
    use crate::drive::glib::OpticalDriveState;

    /// Implementation for [`super::OpticalDriveObject`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::OpticalDriveObject)]
    pub struct OpticalDriveObject {
        /// The name assigned to the drive.
        ///
        /// This will be set to the serial number by default, but can be overwritten by the user.
        #[property(name = "name", get, set, type = String)]
        pub(super) name: RefCell<String>,

        /// The device path of the drive, such as "/dev/sr0".
        #[property(name = "path", get, set, type = String)]
        pub(super) path: RefCell<String>,

        /// The serial number of the optical drive.
        ///
        /// This may be a shortened version of the serial number assigned by the manufacturer.
        #[property(name = "serial-number", get, set, type = String)]
        pub(super) serial_number: RefCell<String>,

        /// The state of the disc in the optical drive.
        #[property(name = "disc-label", get, set, type = String)]
        pub(super) disc_label: RefCell<String>,

        /// The state of the drive.
        #[property(
            name = "drive-state",
            get,
            set,
            type = OpticalDriveState,
            builder(OpticalDriveState::Disconnected))
        ]
        pub(super) drive: Cell<OpticalDriveState>,

        /// Brief description of what caused the failure.
        ///
        /// This is only valid when the state is [`OpticalDriveState::Failed`].
        #[property(name = "error-message", get, set, type = String)]
        pub(super) error_message: RefCell<String>,

        /// The current stage of the copying process.
        ///
        /// This is only valid when the state is [`OpticalDriveState::Copying`].
        #[property(name = "stage", get, set, type = String)]
        pub(super) stage: RefCell<String>,

        /// The length of time the copy operation has been running.
        ///
        /// This is only valid when the state is [`OpticalDriveState::Copying`].
        #[property(name = "elapsed-time", get, set, type = String)]
        pub(super) elapsed_time: RefCell<String>,

        /// The task currently being performed.
        ///
        /// This is only valid when the state is [`OpticalDriveState::Copying`].
        #[property(name = "task", get, set, type = String)]
        pub(super) task: RefCell<String>,

        /// The percent complete (0 -> 0%, 1.0 -> 100%) of the current task.
        ///
        /// This is only valid when the state is [`OpticalDriveState::Copying`].
        #[property(name = "task-progress", get, set, type = f32)]
        pub(super) task_progress: Cell<f32>,

        /// The subtask currently being performed.
        ///
        /// This is only valid when the state is [`OpticalDriveState::Copying`].
        #[property(name = "subtask", get, set, type = String)]
        pub(super) subtask: RefCell<String>,

        /// The percent complete (0 -> 0%, 1.0 -> 100%) of the current subtask.
        ///
        /// This is only valid when the state is [`OpticalDriveState::Copying`].
        #[property(name = "subtask-progress", get, set, type = f32)]
        pub(super) subtask_progress: Cell<f32>,

        /// Interface for communicating with the actor responsible for this drive instance.
        pub(super) handle: RefCell<Option<DriveActorHandle>>,
    }

    impl OpticalDriveObject {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OpticalDriveObject {
        const NAME: &'static str = "OpticalDriveObject";
        type Type = super::OpticalDriveObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for OpticalDriveObject {
    }
}
