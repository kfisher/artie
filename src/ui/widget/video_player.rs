// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the video player widget.
//!
//! The video player widget is used to view the videos in-app to help the user be able to identify
//! additional information about the video such as episode or special feature.

use std::time::Duration;

use gst::{
    ElementFactory,
    SeekFlags,
    State,
};
use gst::prelude::*;

use gtk::{
    Align,
    Box,
    Label,
    Orientation,
    Picture,
    GraphicsOffload,
    GraphicsOffloadEnabled,
    Scale,
};
// use gtk::gdk::{GLContext, Paintable};
use gtk::glib::{self, Object};
use gtk::gdk::Paintable;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::widget::IconButton;

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

        let play_button = IconButton::icon_only("fontawesome.v7.solid.play");
        play_button.set_sensitive(false);
        play_button.add_css_class("default");

        let playbin_element = imp.playbin_element
            .borrow()
            .as_ref()
            .expect("playbin_element was None")
            .clone();
        play_button.connect_clicked(move |_| {
            if let Err(error) = playbin_element.set_state(State::Playing) {
                tracing::error!(?error, "failed to play video");
            }
        });

        let pause_button = IconButton::icon_only("fontawesome.v7.solid.pause");
        pause_button.set_sensitive(false);
        pause_button.add_css_class("default");

        let playbin_element = imp.playbin_element
            .borrow()
            .as_ref()
            .expect("playbin_element was None")
            .clone();
        pause_button.connect_clicked(move |_| {
            if let Err(error) = playbin_element.set_state(State::Paused) {
                tracing::error!(?error, "failed to pause video");
            }
        });

        let stop_button = IconButton::icon_only("fontawesome.v7.solid.stop");
        stop_button.set_sensitive(false);
        stop_button.add_css_class("default");

        let playbin_element = imp.playbin_element
            .borrow()
            .as_ref()
            .expect("playbin_element was None")
            .clone();
        stop_button.connect_clicked(move |_| {
            if let Err(error) = playbin_element.set_state(State::Ready) {
                tracing::error!(?error, "failed to stop video");
            }
        });

        let current_time = Label::builder()
            .label("--:--")
            .build();
        current_time.add_css_class("time-stamp");

        let slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        slider.set_hexpand(true);
        slider.set_draw_value(false);
        slider.set_sensitive(false);
        slider.add_css_class("seek-slider");

        let playbin_element = imp.playbin_element
            .borrow()
            .as_ref()
            .expect("playbin_element was None")
            .clone();
        let video_slider_value_changed = slider.connect_value_changed(move |slider| {
            let value = gst::format::ClockTime::from_seconds(slider.value() as u64);
            if let Err(error) = playbin_element.seek_simple(
                SeekFlags::FLUSH | SeekFlags::KEY_UNIT,
                value,
            ) {
                tracing::error!(?error, "failed to seek");
            }
        });

        let duration_time = Label::builder()
            .label("--:--")
            .build();
        duration_time.add_css_class("time-stamp");

        let controls = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .build();
        controls.append(&play_button);
        controls.append(&pause_button);
        controls.append(&stop_button);
        controls.append(&current_time);
        controls.append(&slider);
        controls.append(&duration_time);
        controls.add_css_class("controls");

        self.append(&graphics_offload);
        self.append(&controls);

        self.set_halign(Align::Center);
        self.set_hexpand(false);
        self.set_orientation(Orientation::Vertical);
        self.set_valign(Align::Start);
        self.set_vexpand(false);

        self.add_css_class("video-player");

        let widget = self.clone();
        glib::spawn_future_local(glib::clone!(
            #[weak]
            widget,
            async move {
                refresh_ui(&widget).await;
            }
        ));

        imp.video_slider.replace(Some(slider));
        imp.video_slider_value_changed.replace(Some(video_slider_value_changed));
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

        let pipeline_bus = playbin_element.bus()
            .expect("failed to get pipeline bus");
        pipeline_bus.add_signal_watch();

        pipeline_bus.connect_message(Some("state-changed"), |_, _msg| {
        });

        let imp = self.imp();
        imp.playbin_element.replace(Some(playbin_element));
        imp.video_sink_element.replace(Some(video_sink_element));
    }

    /// Update the UI based on the current state of the video being played.
    fn refresh_ui(&self) {
        let imp = self.imp();

        let playbin_element = imp.playbin_element
            .borrow()
            .as_ref()
            .expect("playbin_element was None")
            .clone();

        let state = playbin_element.current_state();
        if state != State::Paused && state != State::Playing {
            return;
        }

        let playbin_element = imp.playbin_element
            .borrow()
            .as_ref()
            .expect("playbin_element was None")
            .clone();

        let video_slider = imp.video_slider
            .borrow()
            .as_ref()
            .expect("video_slider was None")
            .clone();

        let duration = playbin_element.query_duration::<gst::format::ClockTime>()
            .expect("duration was None");
        video_slider.set_range(0.0, duration.seconds_f64());

        let position = playbin_element.query_position::<gst::format::ClockTime>()
            .unwrap();

        video_slider.block_signal(
            imp.video_slider_value_changed
                .borrow()
                .as_ref()
                .unwrap()
        );

        video_slider.set_value(position.seconds_f64());

        video_slider.unblock_signal(
            imp.video_slider_value_changed
                .borrow()
                .as_ref()
                .unwrap()
        );
    }
}

/// Task for updating the video player UI.
///
/// This will loop until the application exits.
async fn refresh_ui(widget: &VideoPlayerWidget) {
    loop {
        widget.refresh_ui();
        glib::timeout_future(Duration::from_secs(1)).await;
    }
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gst::{Element, State};
    use gst::prelude::*;

    use gtk::{Box, Scale};

    use gtk::glib::{self, Properties, SignalHandlerId};
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

        // TODO
        pub(super) video_slider: RefCell<Option<Scale>>,

        // TODO
        pub(super) video_slider_value_changed: RefCell<Option<SignalHandlerId>>,
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

        fn dispose(&self) {
            tracing::warn!(">>>>>>>>> DISPOSE");
            if let Some(playbin_element) = self.playbin_element.borrow_mut().as_ref() {
                let _ = playbin_element.set_state(State::Null)
                    .inspect_err(|error| {
                        tracing::warn!(?error, "failed to set state to Null when disposing");
                    });
            }
        }
    }

    impl WidgetImpl for VideoPlayerWidget {}

    impl BoxImpl for VideoPlayerWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
