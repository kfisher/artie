// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::RefCell;

use glib::{self, Properties};

use gtk::{
    Align,
    Box,
    Label,
    Orientation,
    Revealer,
    RevealerTransitionType,
};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::ContextObject;
use crate::ui::widget::DuelIconToggleButton;

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

        obj.append(&self.create_transcoder_view());
        obj.append(&self.create_job_list());
    }

    // TODO
    fn create_transcoder_view(&self) -> Box {
        let view = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        view.add_css_class("transcoder-view");

        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();
        header.add_css_class("header");
        view.append(&header);

        let expand_button = DuelIconToggleButton::builder()
            .active_icon_name("fontawesome.v7.solid.angle-up-symbolic")
            .inactive_icon_name("fontawesome.v7.solid.angle-down-symbolic")
            .build();
        expand_button.add_css_class("default");
        header.append(&expand_button);

        let title = Label::builder()
            .label("Transcoders")
            .hexpand(true)
            .halign(Align::Start)
            .build();
        header.append(&title);

        let revealer = Revealer::builder()
            .reveal_child(true)
            .transition_type(RevealerTransitionType::SlideDown)
            .build();
        revealer.add_css_class("content");
        view.append(&revealer);

        let placeholder = Label::builder()
            .label("TODO: Transcoder status")
            .build();
        revealer.set_child(Some(&placeholder));

        expand_button.bind_property("active", &revealer, "reveal-child")
            .sync_create()
            .build();

        view
    }

    // TODO
    fn create_job_list(&self) -> Box {
        Box::builder()
            .vexpand(true)
            .build()
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

