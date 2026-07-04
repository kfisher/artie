// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use gtk::{Box, prelude::BoxExt};
use glib::{self, Properties};
use gtk::subclass::prelude::*;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::DeleteFormWidget)]
pub struct DeleteFormWidget {
}

impl DeleteFormWidget {
    /// Builds the widget.
    fn build_ui(&self) {
        let label = gtk::Label::builder()
            .label("DeleteForm")
            .build();
        self.obj().append(&label);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for DeleteFormWidget {
    const NAME: &'static str = "ArtieDeleteFormWidget";
    type Type = super::DeleteFormWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for DeleteFormWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for DeleteFormWidget {
}

impl BoxImpl for DeleteFormWidget {
}

