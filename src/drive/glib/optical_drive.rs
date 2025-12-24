// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the GObject for [`OpticalDrive`].

use gtk::glib;
use gtk::glib::Object;
use gtk::subclass::prelude::*;

use crate::drive::OpticalDrive;

glib::wrapper! {
    /// GObject implementation for [`OpticalDrive`].
    pub struct OpticalDriveObject(ObjectSubclass<imp::OpticalDriveObject>);
}

impl OpticalDriveObject {
    /// Creates new [`OpticalDriveObject`] instance from the provided optical drive.
    pub fn new(drive: OpticalDrive) -> Self {
        let obj = Object::builder()
            .property("name", &drive.name)
            .property("path", &drive.path)
            .property("serial_number", &drive.serial_number)
            .property("disc_label", &drive.disc_label())
            .build();
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

    use crate::drive::OpticalDrive;

    /// Implementation for [`super::OpticalDriveObject`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::OpticalDriveObject)]
    pub struct OpticalDriveObject {
        /// Name assigned to the optical drive.
        ///
        /// See Also: [`crate::drive::OpticalDrive::name`].
        #[property(get, set)]
        pub(super) name: RefCell<String>,

        /// The device path of the drive, such as "/dev/sr0".
        ///
        /// See Also: [`crate::drive::OpticalDrive::path`].
        #[property(get, set)]
        pub(super) path: RefCell<String>,

        /// The serial number of the optical drive.
        ///
        /// See Also: [`crate::drive::OpticalDrive::serial_number`].
        #[property(get, set)]
        pub(super) serial_number: RefCell<String>,

        /// The state of the disc in the optical drive.
        ///
        /// See Also: [`crate::drive::OpticalDrive::disc_label`].
        #[property(get, set)]
        pub(super) disc_label: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OpticalDriveObject {
        const NAME: &'static str = "OpticalDriveObject";
        type Type = super::OpticalDriveObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for OpticalDriveObject {}
}
