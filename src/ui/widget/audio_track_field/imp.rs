// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

// use std::cell::RefCell;

use glib::{self, Properties};
use gtk::{Box, Orientation};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::widget::{EntryWidget, DropDownWidget, IconButton};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::AudioTrackFieldWidget)]
pub struct AudioTrackFieldWidget {
}

impl AudioTrackFieldWidget {
    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_orientation(Orientation::Horizontal);
        obj.set_spacing(4);

        let source_track_dropdown = DropDownWidget::builder()
            .label("Source Track")
            .option("(1) Surround Sound - English")
            .build();
        obj.append(&source_track_dropdown);

        let encoder_dropdown = DropDownWidget::builder()
            .label("Encoder")
            .option("Pass-Thru")
            .build();
        obj.append(&encoder_dropdown);

        let name_entry = EntryWidget::builder()
            .label("Name")
            .build();
        obj.append(&name_entry);

        let add_button = IconButton::builder()
            .icon_name("fontawesome.v7.solid.plus")
            .label("Add")
            .secondary_button()
            .build();
        obj.append(&add_button);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for AudioTrackFieldWidget {
    const NAME: &'static str = "ArtieAudioTrackFieldWidget";
    type Type = super::AudioTrackFieldWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for AudioTrackFieldWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for AudioTrackFieldWidget {
}

impl BoxImpl for AudioTrackFieldWidget {
}
