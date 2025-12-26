// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the GObject for [`OpticalDrive`].

use gtk::glib;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::drive::OpticalDrive;

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
            let drive = self.inner.borrow();
            match &drive.disc {
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
