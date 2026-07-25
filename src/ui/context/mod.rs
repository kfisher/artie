// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application context for the UI.
//!
//! [`ContextObject`] provides application data for the UI such as the application mode and handle
//! for interfacing with the message bus.

mod imp;

use std::collections::HashMap;
use std::time::Duration;

use gtk::gio::ListStore;
use gtk::gio::prelude::*;
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;

use crate::Mode;
use crate::bus::Handle;
use crate::drive::{self, OpticalDrive};
use crate::ui::data::{OpticalDriveObject, TranscodeJobObject, TranscoderObject};

glib::wrapper! {
    pub struct ContextObject(ObjectSubclass<imp::ContextObject>);
}

impl ContextObject {
    pub fn new(mode: Mode, bus: Handle) -> Self {
        let obj: Self = Object::builder()
            .property("is-worker", mode == Mode::Worker)
            .build();

        let drive_store = ListStore::new::<OpticalDriveObject>();

        let transcoder_store = ListStore::new::<TranscoderObject>();

        let transcode_job_store = ListStore::new::<TranscodeJobObject>();

        let imp = obj.imp();
        imp.bus.replace(Some(bus.clone()));
        imp.drive_store.replace(Some(drive_store.clone()));
        imp.transcoder_store.replace(Some(transcoder_store.clone()));

        glib::spawn_future_local(glib::clone!(
            #[weak]
            drive_store,
            #[weak]
            transcoder_store,
            #[weak]
            transcode_job_store,
            async move {
                loop {
                    tokio::join!(
                        update_drive_status(&bus, &drive_store),
                        update_transcode_status(&bus, &transcoder_store, &transcode_job_store),
                    );
                    glib::timeout_future(Duration::from_millis(33)).await;
                }
            }
        ));

        obj
    }

    /// Returns the handle for sending messages on the communication bus to the application actors.
    pub fn bus(&self) -> Option<Handle> {
        self.imp().bus.borrow().clone()
    }

    /// Returns list of [`OpticalDriveObject`] instances.
    pub fn drive_store(&self) -> Option<ListStore> {
        self.imp().drive_store
            .borrow()
            .clone()
    }

    /// Returns list of [`TranscoderObject`] instances.
    pub fn transcoder_store(&self) -> Option<ListStore> {
        self.imp().transcoder_store
            .borrow()
            .clone()
    }
}

// TODO: The following update logic could probably be more efficent. Its good enough for now and
//       probably doesn't make a big difference in the grand scheme of things.

/// Update the status of all optical drives in the UI.
async fn update_drive_status(bus: &Handle, store: &ListStore) {
    let serial_numbers = match drive::get_drives(bus).await {
        Ok(serial_numbers) => serial_numbers,
        Err(error) => {
            tracing::error!(?error, "failed to get available drives");
            return;
        },
    };

    let mut drives: Vec<OpticalDrive> = Vec::with_capacity(serial_numbers.len());
    for serial_number in &serial_numbers {
        if let Ok(drive) = drive::get(bus, serial_number).await {
            drives.push(drive);
        };
    }

    update_drive_store(bus, store, drives);
}

/// Update the drive store.
fn update_drive_store(bus: &Handle, store: &ListStore, drives: Vec<OpticalDrive>) {
    let drive_map: HashMap<&String, &OpticalDrive> = drives
        .iter()
        .map(|d| (&d.serial_number, d))
        .collect();

    // First pass: remove items no longer in data or no longer valid objects.
    let mut i = 0;
    while i < store.n_items() {
        let Some(obj) = store.item(i) else {
            tracing::warn!("found none in drive list store");
            store.remove(i);
            continue;
        };
        let Some(obj) = obj.downcast_ref::<OpticalDriveObject>() else {
            tracing::warn!("found invalid object in drive list store");
            store.remove(i);
            continue;
        };
        if drive_map.contains_key(&obj.serial_number()) {
            i += 1;
        } else {
            store.remove(i);
        }
    }

    // Second pass: update existing items and insert new ones in order
    for (desired_pos, drive) in drives.into_iter().enumerate() {
        let desired_pos = desired_pos as u32;
        let current_pos = (desired_pos..store.n_items()).find(|&j| {
            let obj = store.item(j).unwrap();
            let obj = obj.downcast_ref::<OpticalDriveObject>().unwrap();
            obj.serial_number() == drive.serial_number
        });

        match current_pos {
            Some(pos) => {
                let obj = store.item(pos).unwrap();
                let obj = obj.downcast_ref::<OpticalDriveObject>().unwrap();

                obj.update_status(drive);

                if pos != desired_pos {
                    store.remove(pos);
                    store.insert(desired_pos, obj);
                }
            },
            None => {
                let obj = OpticalDriveObject::new(&drive.serial_number, bus.clone());
                store.insert(desired_pos, &obj);
            },
        }
    }
}

/// Update the status of all transcoders in the UI.
async fn update_transcode_status(_bus: &Handle, transcoders: &ListStore, jobs: &ListStore) {
    //--] // TODO: Hard code a few instances for the purposes of setting up the UI.

    if transcoders.n_items() == 0 {
        transcoders.append(&TranscoderObject::new("1"));
        transcoders.append(&TranscoderObject::new("2"));
    }

    if jobs.n_items() == 0 {
        jobs.append(&TranscodeJobObject::new());
        jobs.append(&TranscodeJobObject::new());
        jobs.append(&TranscodeJobObject::new());
        jobs.append(&TranscodeJobObject::new());
        jobs.append(&TranscodeJobObject::new());
        jobs.append(&TranscodeJobObject::new());
    }
}

