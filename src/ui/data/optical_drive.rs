// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of an optical drive.

use std::time::Duration;

use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;

use crate::bus::Handle;
use crate::drive::{self, FormData, FormDataUpdate};
use crate::ui::data::OpticalDriveState;
use crate::models::CopyParamaters;

glib::wrapper! {
    pub struct OpticalDriveObject(ObjectSubclass<imp::OpticalDriveObject>);
}

impl OpticalDriveObject {
    /// Creates a new optical drive object instance.
    ///
    /// # Args
    ///
    /// `serial_number`:  The serial number of the optical drive this object is being created for.
    ///
    /// `bus`:  The bus used for sending messages to the actor, mainly for sending requests to the
    /// actor responsible for managing the associated optical drive.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(serial_number: &str, bus: Handle) -> Self {
        let obj: Self = Object::builder()
            .property("name", serial_number)
            .property("serial-number", serial_number)
            .build();

        let imp = obj.imp();
        imp.bus.replace(Some(bus));

        obj
    }

    /// Cancels a copy operation currently in progress.
    pub async fn cancel_copy_disc(&self) {
        let bus = self.bus();
        let serial_number = self.serial_number();
        if let Err(error) = drive::cancel_copy(&bus, &serial_number).await {
            tracing::error!(sn=serial_number, ?error, "failed to cancel copy disc")
        }
    }

    /// Begin copying the disc in the optical drive.
    ///
    /// # Args
    ///
    /// `params`:  The parameters for the copy operation such as the title, release year, or disc
    /// number of the disc being copied.
    pub async fn copy_disc(&self, params: CopyParamaters) {
        let bus = self.bus();
        let serial_number = self.serial_number();
        if let Err(error) = drive::begin_copy(&bus, &serial_number, params).await {
            tracing::error!(sn=serial_number, ?error, "failed to begin copy");
        }
    }

    /// Get the last saved values for a drive's copy parameters.
    pub async fn read_form_data(&self) -> Option<FormData> {
        let bus = self.bus();
        let serial_number = self.serial_number();
        match drive::read_form_data(&bus, &serial_number).await {
            Ok(data) => Some(data),
            Err(error) => {
                tracing::error!(sn=serial_number, ?error, "failed to read form data");
                None
            },
        }
    }

    /// Reset the optical drive back to the `idle` state.
    pub async fn reset(&self) {
        let bus = self.bus();
        let serial_number = self.serial_number();
        if let Err(error) = drive::reset(&bus, &serial_number).await {
            tracing::error!(sn=serial_number, ?error, "failed to reset drive")
        }
    }

    /// Updates the saved copy parameters for the drive..
    ///
    /// # Args
    ///
    /// `data`:  The updated copy parameters.
    pub async fn save_form_data(&self, data: FormDataUpdate) {
        let bus = self.bus();
        let serial_number = self.serial_number();
        if let Err(error) = drive::save_form_data(&bus, &serial_number, data).await {
            tracing::error!(sn=serial_number, ?error, "failed to save form data")
        }
    }

    /// Updates the status of the drive.
    ///
    /// This will request the current status from the drive's actor instance and then send the
    /// property change notifications if required.
    pub async fn update_status(&self) {
        let bus = self.bus();
        let serial_number = self.serial_number();

        let drive = match drive::get(&bus, &serial_number).await {
            Ok(drive) => drive,
            Err(error) => {
                tracing::error!(sn=serial_number, ?error, "failed to update status");
                return;
            },
        };

        self.set_name(drive.name);
        self.set_path(drive.path);

        match drive.disc {
            crate::drive::DiscState::None => {
                self.set_disc_label(String::default());
            },
            crate::drive::DiscState::Inserted { label, uuid: _ } => {
                self.set_disc_label(label);
            },
        }

        match drive.state {
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

    /// Gets the handle to the application message bus.
    ///
    /// # Panics
    ///
    /// This will panic if the message bus is `None`. This shouldn't be possible given its set when
    /// constructed and never changed.
    fn bus(&self) -> Handle {
        self.imp().bus
            .borrow()
            .as_ref()
            .expect("message bus not set")
            .clone()
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
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::bus::Handle;
    use crate::ui::data::OpticalDriveState;

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
        #[property(name = "serial-number", get, set, type = String, construct_only)]
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

        /// Interface for sending messages to application actors, mainly the actor associated with
        /// this optical drive.
        pub(super) bus: RefCell<Option<Handle>>,
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

#[cfg(test)]
mod tests {
    // TODO
}
