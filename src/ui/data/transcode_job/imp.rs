// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Object implementation.

use std::cell::RefCell;

use gtk::glib::{self, Properties};
// use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::bus::Handle;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::TranscodeJobObject)]
pub struct TranscodeJobObject {
    /// Interface for sending messages to application actors, mainly the actor associated with this
    /// transcoder.
    pub(super) bus: RefCell<Option<Handle>>,
}

impl TranscodeJobObject {
}

#[glib::object_subclass]
impl ObjectSubclass for TranscodeJobObject {
    const NAME: &'static str = "TranscodeJobObject";
    type Type = super::TranscodeJobObject;
}

#[glib::derived_properties]
impl ObjectImpl for TranscodeJobObject {
}

