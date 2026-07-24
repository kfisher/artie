// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::RefCell;

use glib::{self, Properties};

use gtk::{Box, Orientation};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::ContextObject;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::TranscodeQueueWidget)]
pub struct TranscodeQueueWidget {
    /// The application context.
    #[property(get, construct_only)]
    pub(super) context: RefCell<Option<ContextObject>>,
}

impl TranscodeQueueWidget {
    /// TODO
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_hexpand(false);
        obj.set_orientation(Orientation::Vertical);
        obj.add_css_class("transcode-queue");
        obj.set_width_request(300);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for TranscodeQueueWidget {
    const NAME: &'static str = "ArtieTranscodeQueueWidget";
    type Type = super::TranscodeQueueWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for TranscodeQueueWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for TranscodeQueueWidget {
}

impl BoxImpl for TranscodeQueueWidget {
}

