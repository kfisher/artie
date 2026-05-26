// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the video player widget.
//!
//! The video player widget is used to view the videos in-app to help the user be able to identify
//! additional information about the video such as episode or special feature.

use gst::{
    ElementFactory,
};
use gst::prelude::*;

use gtk::{
    Align,
    Orientation,
    Picture,
    GraphicsOffload,
    GraphicsOffloadEnabled,
};
// use gtk::gdk::{GLContext, Paintable};
use gtk::glib::{self, Object};
use gtk::gdk::Paintable;
use gtk::prelude::*;
use gtk::subclass::prelude::*;


glib::wrapper! {
    pub struct VideoPlayerWidget(ObjectSubclass<imp::VideoPlayerWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl VideoPlayerWidget {
    /// Creates a new copy page instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    /// Builds the widget.
    fn build_ui(&self) {
        let imp = self.imp();

        let video_sink_element = imp.video_sink_element
            .borrow()
            .as_ref()
            .expect("video_sink_element was None")
            .clone();

        let paintable = video_sink_element
            .property::<Paintable>("paintable");

        let picture = Picture::builder()
            .paintable(&paintable)
            .build();

        let graphics_offload = GraphicsOffload::builder()
            .black_background(true)
            .enabled(GraphicsOffloadEnabled::Enabled)
            .child(&picture)
            .width_request(720)
            .height_request(480)
            .build();

        self.append(&graphics_offload);

        self.set_halign(Align::Center);
        self.set_hexpand(false);
        self.set_orientation(Orientation::Vertical);
        self.set_valign(Align::Start);
        self.set_vexpand(false);

        self.add_css_class("placeholder");
    }

    /// Initializes the GStreamer pipeline.
    fn init_pipeline(&self) {
        // The default video sink in the playbin element is replaced by the following GTK4
        // paintable sink so that the video can be played within a paintable widget.
        let video_sink_element =  ElementFactory::make("gtk4paintablesink")
            .name("video-sink")
            .build()
            .expect("failed to create video sink element");

        // The playbin element provides an all-in-one abstraction for playing video/audio. It
        // avoids the need to manually create the various audio/video elements while still
        // providing the ability to control subtitles and the selected audio track.
        let playbin_element = ElementFactory::make("playbin3")
            .name("playbin")
            .build()
            .expect("failed to create playbin element");

        playbin_element.set_property("video-sink", &video_sink_element);

        let imp = self.imp();
        imp.playbin_element.replace(Some(playbin_element));
        imp.video_sink_element.replace(Some(video_sink_element));
    }
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gst::{Element};

    use gtk::Box;

    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;

    /// Implemenation for [`super::VideoPlayerWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::VideoPlayerWidget)]
    pub struct VideoPlayerWidget {
        /// Provides an all-in-one abstraction for playing video/audio.
        ///
        /// It avoids the need to manually create the various audio/video elements while still
        /// providing the ability to control subtitles and the selected audio track.
        pub(super) playbin_element: RefCell<Option<Element>>,

        /// Video sink for playing videos within a GTK paintable widget.
        pub(super) video_sink_element: RefCell<Option<Element>>,
    }

    impl VideoPlayerWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoPlayerWidget {
        const NAME: &'static str = "ArtieVideoPlayerWidget";
        type Type = super::VideoPlayerWidget;
        type ParentType = Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for VideoPlayerWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.init_pipeline();
            obj.build_ui();
        }
    }

    impl WidgetImpl for VideoPlayerWidget {}

    impl BoxImpl for VideoPlayerWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
