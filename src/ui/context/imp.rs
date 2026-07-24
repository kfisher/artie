// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Object implementation.

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

    /// List of [`crate::ui::data::TranscoderObject`] instances containing the transcoder data.
    pub(super) transcoder_store: RefCell<Option<ListStore>>,

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
