// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of a transcoder.

use gtk::glib::{self, Object};

mod imp;

glib::wrapper! {
    pub struct TranscoderObject(ObjectSubclass<imp::TranscoderObject>);
}

impl TranscoderObject {
    /// Creates a new transcoder object instance.
    pub fn new(id: &str) -> Self {
        let obj: Self = Object::builder()
            .property("id", id)
            .build();
        obj
    }
}
