// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use gtk::{Box, prelude::BoxExt};
use glib::{self, Properties};
use gtk::subclass::prelude::*;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::TranscodeFormWidget)]
pub struct TranscodeFormWidget {
}

impl TranscodeFormWidget {
    /// Builds the widget.
    fn build_ui(&self) {
        let label = gtk::Label::builder()
            .label("TranscodeForm")
            .build();
        self.obj().append(&label);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for TranscodeFormWidget {
    const NAME: &'static str = "ArtieTranscodeFormWidget";
    type Type = super::TranscodeFormWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for TranscodeFormWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for TranscodeFormWidget {
}

impl BoxImpl for TranscodeFormWidget {
}
