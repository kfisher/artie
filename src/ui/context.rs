// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application context for the UI.
//!
//! [`ContextObject`] provides application data for the UI such as the application mode and handle
//! for interfacing with the message bus.

use gtk::gio::ListStore;
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;

use crate::Mode;
use crate::bus::Handle;
use crate::drive;
use crate::ui::data::OpticalDriveObject;
use crate::task;

glib::wrapper! {
    pub struct ContextObject(ObjectSubclass<imp::ContextObject>);
}

impl ContextObject {
    /// Creates a new context instance.
    pub fn new(mode: Mode, bus: Handle) -> Self {
        let obj: Self = Object::builder()
            .property("is-worker", mode == Mode::Worker)
            .build();


        let drives: Vec<OpticalDriveObject> = match task::block_on(drive::get_drives(&bus)) {
            Ok(drives) => drives.into_iter()
                .map(|serial_number| OpticalDriveObject::new(&serial_number, bus.clone()))
                .collect(),
            Err(error) => {
                tracing::error!(?error, "failed to get drives");
                Vec::default()
            },
        };

        let drive_store = ListStore::from_iter(drives);

        let imp = obj.imp();
        imp.drive_store.replace(Some(drive_store));

        obj
    }

    /// Returns list of [`crate::ui::data::OpticalDriveObject`] instances containing the optical
    /// drive data.
    pub fn drive_store(&self) -> Option<ListStore> {
        self.imp().drive_store
            .borrow()
            .clone()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Properties};
    use gtk::gio::ListStore;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::bus::Handle;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ContextObject)]
    pub struct ContextObject {
        /// List of [`crate::ui::data::OpticalDriveObject`] instances containing the optical
        /// drive data.
        pub(super) drive_store: RefCell<Option<ListStore>>,

        /// Indicates if the application instance is a worker node.
        #[property(name = "is-worker", get, set, type = bool, construct_only)]
        pub(super) is_worker: Cell<bool>,

        /// Message bus for sending requests to the various application actors.
        pub(super) bus: RefCell<Option<Handle>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContextObject {
        const NAME: &'static str = "ContextObject";
        type Type = super::ContextObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ContextObject {
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
