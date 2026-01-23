// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the GObject for [`OpticalDrive`].

use gtk::glib;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::Result;
use crate::drive::{OpticalDrive, OpticalDriveStatus};

glib::wrapper! {
    /// GObject implementation for [`OpticalDrive`].
    pub struct OpticalDriveObject(ObjectSubclass<imp::OpticalDriveObject>);
}

impl OpticalDriveObject {
    /// Creates new [`OpticalDriveObject`] instance from the provided optical drive.
    pub fn new(drive: OpticalDrive) -> Self {
        let obj: Self = Object::builder()
            .build();
        obj.imp().inner.replace(drive);
        obj
    }

    /// Updates the status of the drive.
    ///
    /// This will request the current status from the drive's actor instance and then send the
    /// property change notifications if required.
    pub async fn update_status(&self) {
        let result = self.imp().inner.borrow_mut().update_status().await;
        match result {
            Ok(modified) => {
                if modified {
                    self.notify_disc_label();
                    self.notify_drive_state();
                    self.notify_elapsed_time();
                    self.notify_error_message();
                    self.notify_stage();
                    self.notify_subtask();
                    self.notify_subtask_progress();
                    self.notify_task();
                    self.notify_task_progress();
                }
            },
            Err(error) => {
                tracing::error!(?error, "failed to update status");
            },
        }
    }
}

mod imp {
    //! Implementation for the optical drive object.

    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::drive::{DiscState, OpticalDrive};
    use crate::drive::glib::OpticalDriveState;

    /// Implementation for [`super::OpticalDriveObject`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::OpticalDriveObject)]
    pub struct OpticalDriveObject {
        #[property(name = "name", get, type = String, member = name)]
        #[property(name = "path", get, type = String, member = path)]
        #[property(name = "serial-number", get, type = String, member = serial_number)]
        #[property(name = "disc-label", get = OpticalDriveObject::disc_label, type = String)]
        #[property(
            name = "drive-state",
            get = OpticalDriveObject::drive_state,
            type = OpticalDriveState,
            builder(OpticalDriveState::Disconnected)
        )]
        #[property(
            name = "error-message",
            get = OpticalDriveObject::error_message,
            type = Option<String>
        )]
        #[property(
            name = "stage",
            get = OpticalDriveObject::stage,
            type = Option<String>
        )]
        #[property(
            name = "elapsed-time",
            get = OpticalDriveObject::elapsed_time,
            type = Option<String>
        )]
        #[property(
            name = "task",
            get = OpticalDriveObject::task,
            type = Option<String>
        )]
        #[property(
            name = "task-progress",
            get = OpticalDriveObject::task_progress,
            type = f32
        )]
        #[property(
            name = "subtask",
            get = OpticalDriveObject::subtask,
            type = Option<String>
        )]
        #[property(
            name = "subtask-progress",
            get = OpticalDriveObject::subtask_progress,
            type = f32
        )]
        pub(super) inner: RefCell<OpticalDrive>,
    }

    impl OpticalDriveObject {
        /// Returns the disc label if a disc is inserted into the drive or an empty string if the
        /// optical drive is empty.
        pub fn disc_label(&self) -> String {
            let disc = &self.inner.borrow().disc;
            match disc {
                DiscState::None => String::default(),
                DiscState::Inserted { label, uuid: _ } => label.clone(),
            }
        }

        /// Returns the state of the drive.
        ///
        /// The returned value is the GObject representation of the state that combines the drive 
        /// and disc states into one state enumeration. This provides an value to bind to when
        /// determining which view to display.
        pub fn drive_state(&self) -> OpticalDriveState {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Disconnected => OpticalDriveState::Disconnected,
                crate::drive::OpticalDriveState::Idle => {
                    let disc = &self.inner.borrow().disc;
                    match disc {
                        DiscState::None => OpticalDriveState::Empty,
                        DiscState::Inserted { .. } => OpticalDriveState::Idle,
                    }
                },
                crate::drive::OpticalDriveState::Copying { .. } => OpticalDriveState::Copying,
                crate::drive::OpticalDriveState::Success => OpticalDriveState::Success,
                crate::drive::OpticalDriveState::Failed { .. } => OpticalDriveState::Failed,
            }
        }

        /// Returns the elapsed time of the copy operation or `None` if the drive is not in the
        /// copying state.
        pub fn elapsed_time(&self) -> Option<String> {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Copying { elapsed_time, .. } => {
                    let total_seconds = elapsed_time.as_secs();
                    let hours = total_seconds / 3600;
                    let minutes = (total_seconds % 3600) / 60;
                    let seconds = total_seconds % 60;
                    Some(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
                },
                _ => None,
            }

        }

        /// Returns the error message.
        ///
        /// The result will be `Some` if the drive is in the failed state and `None` if in any
        /// other state.
        pub fn error_message(&self) -> Option<String> {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Failed { error } => Some(error.clone()),
                _ => None
            }
        }

        /// Returns the stage of the copy operation or `None` if the drive is in a state other than
        /// the copying state.
        pub fn stage(&self) -> Option<String> {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Copying { stage, .. } => Some(stage.clone()),
                _ => None,
            }
        }

        /// Returns the label for the current task of the copy operation or `None` if the drive is
        /// not in the copying state.
        pub fn task(&self) -> Option<String> {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Copying { task, .. } => Some(task.clone()),
                _ => None,
            }
        }

        /// Returns the progress of the current task.
        ///
        /// This value will be between 0 and 1 if the drive is in the copying state based on the
        /// last progress update. If not in the copying state, will return zero.
        pub fn task_progress(&self) -> f32 {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Copying { task_progress, .. } => *task_progress,
                _ => 0.0,
            }
        }

        /// Returns the label for the current subtask of the copy operation or `None` if the drive
        /// is not in the copying state.
        pub fn subtask(&self) -> Option<String> {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Copying { subtask, .. } => Some(subtask.clone()),
                _ => None,
            }
        }

        /// Returns the progress of the current subtask.
        ///
        /// This value will be between 0 and 1 if the drive is in the copying state based on the
        /// last progress update. If not in the copying state, will return zero.
        pub fn subtask_progress(&self) -> f32 {
            let state = &self.inner.borrow().state;
            match state {
                crate::drive::OpticalDriveState::Copying { subtask_progress, .. } => {
                    *subtask_progress
                },
                _ => 0.0,
            }
        }
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
