// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Object implementation.

use std::cell::RefCell;

use gtk::glib::{self, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::bus::Handle;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::TranscoderObject)]
pub struct TranscoderObject {
    /// The transcoder's identifier.
    #[property(get, set)]
    pub(super) id: RefCell<String>,

    /// Interface for sending messages to application actors, mainly the actor associated with this
    /// transcoder.
    pub(super) bus: RefCell<Option<Handle>>,
}

impl TranscoderObject {
}

#[glib::object_subclass]
impl ObjectSubclass for TranscoderObject {
    const NAME: &'static str = "TranscoderObject";
    type Type = super::TranscoderObject;
}

#[glib::derived_properties]
impl ObjectImpl for TranscoderObject {
}
