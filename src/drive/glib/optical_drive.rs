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

    use std::cell::RefCell;
    use std::rc::Rc;

    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::drive::{DiscState, OpticalDrive};

    /// Implementation for [`super::OpticalDriveObject`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::OpticalDriveObject)]
    pub struct OpticalDriveObject {
        #[property(name = "name", get, type = String, member = name)]
        #[property(name = "path", get, type = String, member = path)]
        #[property(name = "serial-number", get, type = String, member = serial_number)]
        #[property(name = "disc-label", get = OpticalDriveObject::disc_label, type = String)]
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
