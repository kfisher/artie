// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Object implementation.

use std::cell::{Cell, RefCell};

use gtk::glib::{self, GString, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::data::AudioTrackObject;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::AudioEncodeOptionObject)]
pub struct AudioEncodeOptionObject {
    /// The track number.
    ///
    /// Track numbers start at one, not zero like indexes into an array.
    #[property(get, set)]
    pub track_number: Cell<u8>,

    /// The source track that is being encoded.
    #[property(get, set)]
    pub source_track: RefCell<Option<AudioTrackObject>>,

    /// The encoder that will be used to encode the track.
    #[property(get, set)]
    pub encoder: RefCell<GString>,

    /// The name of the track.
    #[property(get, set)]
    pub name: RefCell<GString>,
}

#[glib::object_subclass]
impl ObjectSubclass for AudioEncodeOptionObject {
    const NAME: &'static str = "ArtieAudioEncodeOptionObject";
    type Type = super::AudioEncodeOptionObject;
}

#[glib::derived_properties]
impl ObjectImpl for AudioEncodeOptionObject {
}
