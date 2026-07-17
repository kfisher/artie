// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::{Cell, RefCell};

use glib::{self, Object, Properties};
use gtk::{
    Align,
    Box,
    Label,
    ListItem,
    Orientation,
    SignalListItemFactory,
};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

// use handbrake;

use crate::ui::data::{
    VideoObject,
};

/// The maximum number of subtitle tracks that can be added.
///
/// This isn't based on limits from HandBrake or any video container format which may not have a
/// limit. 100 is far greater then the number of tracks a reasonable person would want to add.
const MAX_TRACK_COUNT: u32 = 100;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::SubtitleTrackFieldWidget)]
pub struct SubtitleTrackFieldWidget {
    /// The active video.
    #[property(get, set = Self::set_video, nullable)]
    pub(super) video: RefCell<Option<VideoObject>>,

    /// Indicates if the field is valid.
    #[property(get)]
    pub(super) is_valid: Cell<bool>,
}

impl SubtitleTrackFieldWidget {
    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(4);
        obj.set_vexpand(true);
    }

    /// Sets the valid/invalid status of the field.
    fn set_is_valid(&self, value: bool) {
        if self.is_valid.get() != value {
            self.is_valid.set(value);
            self.obj().notify_is_valid();
        }
    }

    /// Setter for the active video.
    ///
    /// If `video` is `Some`, the form will be updated to reflect the provided video. If `None`,
    /// the form's values will be reset back to default. In both cases, any user provided changes
    /// will be reset.
    fn set_video(&self, video: Option<VideoObject>) {
        let Some(video) = video else {
            return;
        };

        // Replacement must occur here due to the following triggering events that expect to have
        // the video set.
        self.video.replace(Some(video));
        self.validate();
    }

    /// Updates the "is-valid" property based on the current values of the field.
    fn validate(&self) {
        self.set_is_valid(self.video.borrow().is_some());
    }
}

#[glib::object_subclass]
impl ObjectSubclass for SubtitleTrackFieldWidget {
    const NAME: &'static str = "ArtieSubtitleTrackFieldWidget";
    type Type = super::SubtitleTrackFieldWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for SubtitleTrackFieldWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for SubtitleTrackFieldWidget {
}

impl BoxImpl for SubtitleTrackFieldWidget {
}

/// Function used as the callback when setting up a column in a column view when the column
/// contains a label.
///
/// `factory` is unused and `obj` is expected to be a [`ListItem`] object.
fn label_column_setup(_factory: &SignalListItemFactory, obj: &Object) {
    let label = Label::builder()
        .halign(Align::Start)
        .hexpand(true)
        .build();
    obj.downcast_ref::<ListItem>()
        .unwrap()
        .set_child(Some(&label));
}
