// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the video player widget.
//!
//! The video player widget is used to view the videos in-app to help the user be able to identify
//! additional information about the video such as episode or special feature.

use std::time::Duration;

use gst::{
    Element,
    ElementFactory,
    MessageView,
    SeekFlags,
    State,
    StreamCollection,
    StreamType,
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
use gtk::glib::{self, FlagsClass, Object, SignalHandlerId};
use gtk::gdk::Paintable;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use tokio::sync::mpsc;

use crate::ui::helpers;
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
            .width_request(853)
            .height_request(480)
            .build();

        let play_button = IconButton::icon_only("fontawesome.v7.solid.play");
        play_button.set_sensitive(false);
        play_button.add_css_class("default");

        let video_player = self.clone();
        play_button.connect_clicked(move |_| video_player.play());

        let pause_button = IconButton::icon_only("fontawesome.v7.solid.pause");
        pause_button.set_sensitive(false);
        pause_button.add_css_class("default");

        let video_player = self.clone();
        pause_button.connect_clicked(move |_| video_player.pause());

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

        self.add_css_class("video-player-widget");

        let widget = self.clone();
        glib::spawn_future_local(glib::clone!(
            #[weak]
            widget,
            async move {
                refresh_ui(&widget).await;
            }
        ));

        imp.pause_button.replace(pause_button);
        imp.play_button.replace(play_button);
        imp.video_slider.replace(slider);
        imp.video_slider_value_changed.replace(Some(video_slider_value_changed));
        imp.duration_label.replace(duration_time);
        imp.position_label.replace(current_time);
    }

    /// Called when the video changes
    fn on_video_changed(&self) {
        let imp = self.imp();

        let playbin_element = imp.playbin_element
            .borrow()
            .clone();
        let Some(playbin_element) = playbin_element else {
            tracing::error!("playbin_element was None");
            return;
        };

        if let Err(error) = playbin_element.set_state(State::Null) {
            tracing::error!(?error, "failed to reset video");
            return;
        }

        self.set_controls_enabled(false);

        imp.position_label.borrow().set_text("--:--");
        imp.duration_label.borrow().set_text("--:--");

        let video = imp.video
            .borrow()
            .clone();

        let Some(video) = video else {
            return;
        };

        let path = std::path::PathBuf::from(video.path());
        if !path.is_file() {
            tracing::error!(?path, "path does not exist");
            return;
        }

        let path = format!("file://{0}", video.path());
        playbin_element.set_property("uri", path);

        self.pause();
    }

    /// Pause the video.
    fn pause(&self) {
        let imp = self.imp();

        let playbin_element = imp.playbin_element
            .borrow()
            .clone();
        let Some(playbin_element) = playbin_element else {
            tracing::error!("playbin_element was None");
            return;
        };

        if let Err(error) = playbin_element.set_state(State::Paused) {
            tracing::error!(?error, "failed to pause video");
        }

        imp.pause_button.borrow().set_visible(false);
        imp.play_button.borrow().set_visible(true);
    }

    /// Play the video.
    fn play(&self) {
        let imp = self.imp();

        let playbin_element = imp.playbin_element
            .borrow()
            .clone();
        let Some(playbin_element) = playbin_element else {
            tracing::error!("playbin_element was None");
            return;
        };

        if let Err(error) = playbin_element.set_state(State::Playing) {
            tracing::error!(?error, "failed to play video");
        }

        imp.pause_button.borrow().set_visible(true);
        imp.play_button.borrow().set_visible(false);
    }

    /// Enabled or disable the controls.
    ///
    /// # Args
    ///
    /// `enabled`  Indicates if the player controls should be enabled or disabled.
    fn set_controls_enabled(&self, enabled: bool) {
        let imp = self.imp();

        imp.play_button
            .borrow()
            .set_sensitive(enabled);

        imp.pause_button
            .borrow()
            .set_sensitive(enabled);

        imp.video_slider
            .borrow()
            .set_sensitive(enabled);
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

        pipeline_bus.connect_message(Some("error"), move |_bus, msg| {
            let MessageView::Error(error) = msg.view() else {
                tracing::warn!(view=?msg.view(), "unexpected message type");
                return;
            };

            let src = error.src();
            let err = error.error();
            tracing::error!(error=?err, ?src, "received gstreamer error");
        });

        let playbin_element_clone = playbin_element.clone();
        pipeline_bus.connect_message(Some("state-changed"), move |_bus, msg| {
            if msg.src().map(|src| src != &playbin_element_clone).unwrap_or(true) {
                return;
            }

            let MessageView::StateChanged(state_change) = msg.view() else {
                tracing::warn!(view=?msg.view(), "unexpected message type");
                return;
            };

            if state_change.current() != State::Playing {
                return;
            }

            // tracing::info!(">>>>>>> STATE CHANGE");
        });

        // connect_message requires Send trait which is not supported by GObjects which means that
        // we can't pass a clone. Instead, use channels to relay the messages.
        let (tx, mut rx) = mpsc::channel(5);
        let video_player = self.clone();
        pipeline_bus.connect_message(Some("stream-collection"), move |_bus, msg| {
            let MessageView::StreamCollection(stream_collection) = msg.view() else {
                tracing::warn!(view=?msg.view(), "unexpected message type");
                return;
            };

            let stream_collection = stream_collection.stream_collection();
            if let Err(error) = tx.blocking_send(stream_collection) {
                tracing::error!(?error, "failed to send stream collection");
            }
        });
        glib::spawn_future_local(glib::clone!(
            #[weak]
            video_player,
            async move {
                while let Some(stream_collection) = rx.recv().await {
                    video_player.update_stream_data(&stream_collection);
                }
            }
        ));

        pipeline_bus.add_signal_watch();

        let flags = playbin_element.property_value("flags");

        let flags_class = FlagsClass::with_type(flags.type_())
            .expect("flags_class was None");

        let flags = flags_class.builder_with_value(flags)
            .expect("flags builder was None")
            .set_by_nick("audio")
            .set_by_nick("video")
            .unset_by_nick("text")
            .build()
            .expect("built flags was None");

        playbin_element.set_property_from_value("flags", &flags);

        let imp = self.imp();
        imp.playbin_element.replace(Some(playbin_element));
        imp.video_sink_element.replace(Some(video_sink_element));
    }

    /// Update the UI based on the current state of the video being played.
    fn refresh_ui(&self) {
        let imp = self.imp();

        let playbin_element_cell = imp.playbin_element.borrow();
        let Some(playbin_element) = playbin_element_cell.clone() else {
            tracing::error!("playbin_element was None");
            return;
        };
        drop(playbin_element_cell);

        let state = playbin_element.current_state();
        if state == State::Paused {
            imp.pause_button.borrow().set_visible(false);
            imp.play_button.borrow().set_visible(true);
        } else if state == State::Playing {
            imp.pause_button.borrow().set_visible(true);
            imp.play_button.borrow().set_visible(false);
        } else {
            return;
        }

        update_time(
            imp.position_label.borrow().as_ref(),
            imp.duration_label.borrow().as_ref(), 
            imp.video_slider.borrow().as_ref(),
            &imp.video_slider_value_changed
                .borrow()
                .as_ref()
                .unwrap(),
            &playbin_element
        );
    }

    /// Updates the video, audio, and subtitle stream information.
    ///
    /// # Args
    ///
    /// `stream_collection`  The stream data reported by GStreamer for the video currently being
    /// played.
    fn update_stream_data(&self, stream_collection: &StreamCollection) {
        for stream in stream_collection {
            let Some(_id) = stream.stream_id() else {
                continue;
            };

            match stream.stream_type() {
                StreamType::VIDEO => {
                    tracing::info!("TODO: handle video stream")
                },
                StreamType::AUDIO => {
                    tracing::info!("TODO: handle audio stream")
                },
                StreamType::TEXT => {
                    tracing::info!("TODO: handle subtitle stream")
                },
                _ => {
                    tracing::warn!(type=?stream.stream_type(), "unexpected stream type");
                }
            }
        }
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

/// Update the various time widgets in the video player.
///
/// # Args
///
/// `position_label`  The label that displays the current position in the video.
///
/// `duration_label`  The label that displays the duration of the video.
///
/// `video_slider`  The slider for seeking thru the video.
///
/// `video_slider_change_signal`  The signal id for the signal handler id for the callback when the
/// video slider changes value. This signal callback needs to be blocked when the slider is updated
/// from the video stream.
///
/// `playbin_element`  The GStreamer primary element.
fn update_time(
    position_label: &Label,
    duration_label: &Label,
    video_slider: &Scale,
    slider_change_signal: &SignalHandlerId,
    playbin_element: &Element
) {
    let Some(duration) = playbin_element.query_duration::<gst::format::ClockTime>() else {
        tracing::warn!("unable to get duration");
        return;
    };

    let Some(position) = playbin_element.query_position::<gst::format::ClockTime>() else {
        tracing::warn!("unable to get position");
        return;
    };

    video_slider.block_signal(slider_change_signal);
    video_slider.set_range(0.0, duration.seconds_f64());
    video_slider.set_value(position.seconds_f64());
    video_slider.unblock_signal(slider_change_signal);

    position_label.set_text(&helpers::format_duration_secs(position.seconds()));
    duration_label.set_text(&helpers::format_duration_secs(duration.seconds()));
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gst::{Element, State};
    use gst::prelude::*;

    use gtk::{Box, Label, Scale};

    use gtk::glib::{self, Properties, SignalHandlerId};
    use gtk::subclass::prelude::*;

    use crate::ui::data::VideoObject;
    use crate::ui::widget::IconButton;

    /// Implemenation for [`super::VideoPlayerWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::VideoPlayerWidget)]
    pub struct VideoPlayerWidget {
        /// The active video.
        #[property(get, set = Self::set_video, nullable)]
        pub(super) video: RefCell<Option<VideoObject>>,

        /// Provides an all-in-one abstraction for playing video/audio.
        ///
        /// It avoids the need to manually create the various audio/video elements while still
        /// providing the ability to control subtitles and the selected audio track.
        pub(super) playbin_element: RefCell<Option<Element>>,

        /// The button used to pause the video.
        pub(super) pause_button: RefCell<IconButton>,

        /// The button used to play the video.
        pub(super) play_button: RefCell<IconButton>,

        /// The path to the video.
        pub(super) video_path: RefCell<Option<String>>,

        /// Video sink for playing videos within a GTK paintable widget.
        pub(super) video_sink_element: RefCell<Option<Element>>,

        /// Slider used to indicate the current position in the video and can be used to seek to
        /// thru the video.
        pub(super) video_slider: RefCell<Scale>,

        /// Signal handler identifier for the callback when the video slider's value changes. This
        /// is used to block the callback when the value changes due to the playback.
        pub(super) video_slider_value_changed: RefCell<Option<SignalHandlerId>>,

        /// Label used to display the current position in the video.
        pub(super) position_label: RefCell<Label>,

        /// Label used to display the duration of the video.
        pub(super) duration_label: RefCell<Label>,
    }

    impl VideoPlayerWidget {
        fn set_video(&self, video: Option<VideoObject>) {
            self.video.replace(video);
            self.obj().on_video_changed();
        }
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
