// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of a transcode job.

use gtk::glib::{self, Object};

mod imp;

glib::wrapper! {
    pub struct TranscodeJobObject(ObjectSubclass<imp::TranscodeJobObject>);
}

impl TranscodeJobObject {
    /// Creates a new transcode job object instance.
    pub fn new() -> Self {
        let obj: Self = Object::builder()
            .build();
        obj
    }
}

