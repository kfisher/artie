// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::RefCell;

use glib::{self, Properties};
use gtk::{Align, Box, DropDown, Label, StringList, Orientation};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::DropDownWidget)]
pub struct DropDownWidget {
    /// The label for the dropdown.
    #[property(get, set)]
    pub(super) label: RefCell<Option<String>>,

    /// The dropdown option model.
    #[property(get, set)]
    pub(super) model: RefCell<StringList>,

    /// The underlying [`DropDown`] GTK widget.
    pub(super) dropdown: RefCell<DropDown>,
}

impl DropDownWidget {
    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_orientation(Orientation::Vertical);

        let dropdown = DropDown::builder()
            .build();
        obj.bind_property("model", &dropdown, "model")
            .sync_create()
            .build();
        obj.append(&dropdown);

        let label = Label::builder()
            .halign(Align::Start)
            .build();
        label.add_css_class("field-label");
        label.add_css_class("bottom");
        obj.bind_property("label", &label, "label")
            .sync_create()
            .build();
        obj.append(&label);

        self.dropdown.replace(dropdown);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for DropDownWidget {
    const NAME: &'static str = "ArtieDropDownWidget";
    type Type = super::DropDownWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for DropDownWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for DropDownWidget {
}

impl BoxImpl for DropDownWidget {
}
