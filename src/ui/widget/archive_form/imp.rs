// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use gtk::{Box, prelude::BoxExt};
use glib::{self, Properties};
use gtk::subclass::prelude::*;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::ArchiveFormWidget)]
pub struct ArchiveFormWidget {
}

impl ArchiveFormWidget {
    /// Builds the widget.
    fn build_ui(&self) {
        let label = gtk::Label::builder()
            .label("ArchiveForm")
            .build();
        self.obj().append(&label);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ArchiveFormWidget {
    const NAME: &'static str = "ArtieArchiveFormWidget";
    type Type = super::ArchiveFormWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for ArchiveFormWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for ArchiveFormWidget {
}

impl BoxImpl for ArchiveFormWidget {
}

