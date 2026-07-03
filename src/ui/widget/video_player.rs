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
use gst::event::SelectStreams;
use gst::prelude::*;

use gtk::{
    Align,
    Box,
    CheckButton,
    ColumnView,
    ColumnViewColumn,
    DropDown,
    Label,
    ListItem,
    Orientation,
    Picture,
    GraphicsOffload,
    GraphicsOffloadEnabled,
    NoSelection,
    Scale,
    SignalListItemFactory,
    SingleSelection,
    StringList,
};
use gtk::gio::ListStore;
use gtk::glib::{self, FlagsClass, Object, SignalHandlerId};
use gtk::gdk::Paintable;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use tokio::sync::mpsc;

use crate::ui::data::{AudioTrackObject, SubtitleTrackObject, VideoObject, VideoTrackObject};
use crate::ui::helpers;
use crate::ui::widget::{DuelIconToggleButton, IconToggleButton};

/// The fixed width, in pixels, that video playback is displayed at.
const VIDEO_FRAME_WIDTH: i32 = 720;

/// The fixed height, in pixels, that video playback is displayed at.
const VIDEO_FRAME_HEIGHT: i32 = (VIDEO_FRAME_WIDTH as f32 / (16.0 / 9.0)) as i32;

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

    /// Bind to the selected video.
    fn bind_video(&self, video: &VideoObject) {
        let mut video_signals = self.imp().video_signals.borrow_mut();

        let video_player = self.clone();
        let preview_changed = video.connect_closure(
            "preview-changed",
            false,
            glib::closure_local!(move |_video: VideoObject| {
                video_player.change_preview_tracks();
            }),
        );
        video_signals.push(preview_changed);
    }

    fn change_preview_tracks(&self) {
        // If the controls aren't enabled, then the video isn't ready yet or is invalid. In either
        // case, ignore this signal instance.
        if !self.controls_enabled() {
            return;
        }

        let imp = self.imp();

        let Some(playbin_element) = imp.playbin_element.borrow().clone() else {
            return;
        };

        let video = imp.video
            .borrow()
            .clone();
        let Some(video) = video else {
            return;
        };

        let streams = video.get_selected_preview_tracks()
            .gstreamer_identifiers();
        tracing::trace!(?streams, "change selected tracks");

        let event = SelectStreams::builder(streams.iter().map(|s| s.as_ref()))
            .build();
        playbin_element.send_event(event);
    }

    /// Builds the widget.
    fn build_ui(&self) {
        self.set_orientation(Orientation::Vertical);
        self.set_halign(Align::Start);
        self.set_hexpand(false);
        self.add_css_class("video-player");

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
            .width_request(VIDEO_FRAME_WIDTH)
            .height_request(VIDEO_FRAME_HEIGHT)
            .build();
        self.append(&graphics_offload);

        let seek_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        seek_row.add_css_class("controls");
        self.append(&seek_row);

        let current_time = Label::builder()
            .label("--:--")
            .build();
        current_time.add_css_class("time-stamp");
        seek_row.append(&current_time);

        let slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        slider.set_hexpand(true);
        slider.set_draw_value(false);
        slider.set_sensitive(false);
        slider.add_css_class("seek-slider");
        seek_row.append(&slider);

        self.bind_property("controls-enabled", &slider, "sensitive")
            .sync_create()
            .build();

        let this = self;
        let video_slider_value_changed = slider.connect_value_changed(glib::clone!(
            #[weak]
            this,
            move |slider| {
                let value = slider.value();
                this.video_seek(value as u64);
            }
        ));

        let duration_time = Label::builder()
            .label("--:--")
            .build();
        duration_time.add_css_class("time-stamp");
        seek_row.append(&duration_time);

        let controls = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .build();
        controls.add_css_class("controls");
        self.append(&controls);

        let play_pause_button = DuelIconToggleButton::builder()
            .active_icon_name("fontawesome.v7.solid.pause")
            .inactive_icon_name("fontawesome.v7.solid.play")
            .no_highlight()
            .build();
        play_pause_button.add_css_class("default");
        controls.append(&play_pause_button);

        let this = self;
        play_pause_button.connect_active_notify(glib::clone!(
            #[weak]
            this,
            move |button| {
                if button.is_active() {
                    this.play();
                } else {
                    this.pause();
                }
            }
        ));

        self.bind_property("controls-enabled", &play_pause_button, "sensitive")
            .sync_create()
            .build();

        let spacer = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true)
            .build();
        controls.append(&spacer);

        let audio_controls = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        audio_controls.add_css_class("track-controls");
        controls.append(&audio_controls);

        let audio_button = DuelIconToggleButton::builder()
            .active()
            .active_icon_name("fontawesome.v7.solid.volume")
            .inactive_icon_name("fontawesome.v7.solid.volume-xmark")
            .no_highlight()
            .build();
        audio_button.add_css_class("default");
        audio_button.set_sensitive(false);
        audio_controls.append(&audio_button);

        self.bind_property("controls-enabled", &audio_button, "sensitive")
            .sync_create()
            .build();

        let this = self;
        audio_button.connect_clicked(glib::clone!(
            #[weak]
            this,
            move |button| {
                this.set_mute(!button.is_active());
            }
        ));

        let divider = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        divider.add_css_class("vertical-divider");
        audio_controls.append(&divider);

        let audio_options = StringList::default();
        let audio_dropdown = DropDown::builder()
            .model(&audio_options)
            .build();
        audio_dropdown.set_sensitive(false);
        audio_controls.append(&audio_dropdown);

        self.bind_property("controls-enabled", &audio_dropdown, "sensitive")
            .sync_create()
            .build();

        let this = self;
        audio_dropdown.connect_selected_item_notify(glib::clone!(
            #[weak]
            this,
            move |dropdown| {
                this.select_audio_track(dropdown.selected());
            }
        ));

        let cc_controls = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        cc_controls.add_css_class("track-controls");
        controls.append(&cc_controls);

        let cc_button = DuelIconToggleButton::builder()
            .active_icon_name("fontawesome.v7.solid.closed-captioning")
            .inactive_icon_name("fontawesome.v7.regular.closed-captioning")
            .no_highlight()
            .build();
        cc_button.set_sensitive(false);
        cc_button.add_css_class("default");
        cc_controls.append(&cc_button);

        self.bind_property("controls-enabled", &cc_button, "sensitive")
            .sync_create()
            .build();

        let this = self;
        cc_button.connect_clicked(glib::clone!(
            #[weak]
            this,
            move |button| {
                this.enable_subtitles(button.is_active());
            }
        ));

        let divider = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        divider.add_css_class("vertical-divider");
        cc_controls.append(&divider);

        let cc_options = StringList::default();
        let cc_dropdown = DropDown::builder()
            .model(&cc_options)
            .build();
        cc_dropdown.set_sensitive(false);
        cc_controls.append(&cc_dropdown);

        self.bind_property("controls-enabled", &cc_dropdown, "sensitive")
            .sync_create()
            .build();
        let this = self;
        cc_dropdown.connect_selected_item_notify(glib::clone!(
            #[weak]
            this,
            move |dropdown| {
                this.select_subtitle_track(dropdown.selected());
            }
        ));

        imp.duration_label.replace(duration_time);
        imp.play_pause_button.replace(play_pause_button);
        imp.position_label.replace(current_time);
        imp.video_slider.replace(slider);
        imp.video_slider_value_changed.replace(Some(video_slider_value_changed));
        imp.audio_track_options.replace(audio_options);
        imp.subtitle_track_options.replace(cc_options);

        let this = self;
        glib::spawn_future_local(glib::clone!(
            #[weak]
            this,
            async move {
                refresh_ui(&this).await;
            }
        ));
    }

    /// Enables or disables the display of subtitles.
    pub fn enable_subtitles(&self, enabled: bool) {
        let playbin_element = self.imp().playbin_element
            .borrow()
            .clone();
        let Some(playbin_element) = playbin_element else {
            return;
        };

        enable_subtitles(&playbin_element, enabled);
    }

    /// Initializes the GStreamer pipeline.
    fn init_pipeline(&self) {
        // The default video sink in the playbin element is replaced by the following GTK4
        // paintable sink so that the video can be played within a paintable widget.
        let video_sink_element =  ElementFactory::make("gtk4paintablesink")
            .name("video-sink")
            .build()
            .expect("failed to create video sink element");

        let video_filter_element = video_filter();

        // The playbin element provides an all-in-one abstraction for playing video/audio. It
        // avoids the need to manually create the various audio/video elements while still
        // providing the ability to control subtitles and the selected audio track.
        let playbin_element = ElementFactory::make("playbin3")
            .name("playbin")
            .build()
            .expect("failed to create playbin element");
        playbin_element.set_property("mute", false);
        playbin_element.set_property("video-sink", &video_sink_element);
        playbin_element.set_property("video-filter", &video_filter_element);

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

        // connect_message requires Send trait which is not supported by GObjects which means that
        // we can't pass a clone. Instead, use channels to relay the messages.
        let (tx, mut rx) = mpsc::channel(5);

        let playbin_element_clone = playbin_element.clone();
        let eos_tx = tx.clone();
        pipeline_bus.connect_message(
            Some("eos"),
            move |_bus, msg| {
                if msg.src().map(|src| src != &playbin_element_clone).unwrap_or(true) {
                    return;
                }

                let MessageView::Eos(_) = msg.view() else {
                    tracing::warn!(view=?msg.view(), "unexpected message type");
                    return;
                };

                if let Err(error) = eos_tx.blocking_send(GstMessage::Eos) {
                    tracing::error!(?error, "failed to send stream collection");
                }
            });
        pipeline_bus.connect_message(
            Some("stream-collection"),
            move |_bus, msg| {
                let MessageView::StreamCollection(stream_collection) = msg.view() else {
                    tracing::warn!(view=?msg.view(), "unexpected message type");
                    return;
                };

                let payload = GstMessage::StreamCollection(stream_collection.stream_collection());
                if let Err(error) = tx.blocking_send(payload) {
                    tracing::error!(?error, "failed to send stream collection");
                }
            });

        let video_player = self.clone();
        glib::spawn_future_local(glib::clone!(
            #[weak]
            video_player,
            async move {
                while let Some(msg) = rx.recv().await {
                    match msg {
                        GstMessage::Eos => {
                            video_player.video_ended();
                        },
                        GstMessage::StreamCollection(stream_collection) => {
                            video_player.update_stream_data(&stream_collection);
                        },
                    }
                }
            }
        ));

        pipeline_bus.add_signal_watch();

        // The CC button toggle will be initialized as inactive, so insure the pipeline reflects
        // that initial value.
        enable_subtitles(&playbin_element, false);

        let imp = self.imp();
        imp.playbin_element.replace(Some(playbin_element));
        imp.video_sink_element.replace(Some(video_sink_element));
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

        video.reset_preview_data();

        let path = std::path::PathBuf::from(video.path());
        if !path.is_file() {
            tracing::error!(?path, "path does not exist");
            return;
        }

        self.bind_video(&video);

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
            imp.play_pause_button
                .borrow()
                .set_active(false);
        } else if state == State::Playing {
            imp.play_pause_button
                .borrow()
                .set_active(true);
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

    /// Set the selected audio track.
    ///
    /// # Args
    ///
    /// `index`  The index of the audio track. This is the zero-index value.
    fn select_audio_track(&self, index: u32) {
        let imp = self.imp();
        let video = imp.video
            .borrow()
            .clone();
        if let Some(video) = video {
            if let Some(track) = video.get_audio_track(index) {
                let preview = track.preview();
                preview.set_selected(true);
            }
        }
    }

    /// Set the selected subtitle track.
    ///
    /// # Args
    ///
    /// `index`  The index of the subtitle track. This is the zero-index value.
    fn select_subtitle_track(&self, index: u32) {
        let imp = self.imp();
        let video = imp.video
            .borrow()
            .clone();
        if let Some(video) = video {
            if let Some(track) = video.get_subtitle_track(index) {
                let preview = track.preview();
                preview.set_selected(true);
            }
        }
    }

    /// Toggle mute on and off.
    ///
    /// # Args
    ///
    /// `mute`:  Indicates if the audio should be muted or unmuted.
    pub fn set_mute(&self, mute: bool) {
        let playbin_element = self.imp().playbin_element
            .borrow()
            .clone();
        let Some(playbin_element) = playbin_element else {
            return;
        };

        playbin_element.set_property("mute", mute);
    }

    /// Updates the video, audio, and subtitle stream information.
    ///
    /// # Args
    ///
    /// `stream_collection`  The stream data reported by GStreamer for the video currently being
    /// played.
    fn update_stream_data(&self, stream_collection: &StreamCollection) {
        let video = self.imp().video
            .borrow()
            .clone();
        let Some(video) = video else {
            return;
        };

        let mut audio_track_index = 0;
        let mut subtitle_track_index = 0;
        let mut video_track_index = 0;
        for (_index, stream) in stream_collection.iter().enumerate() {
            let Some(_id) = stream.stream_id() else {
                continue;
            };

            // TODO: The container_index has been added to the track data. Double check that here
            //       or even add an API to video to use container index.

            match stream.stream_type() {
                StreamType::VIDEO => {
                    // There will generally only ever be one video track, but the MKV format
                    // supports more than one. Not sure if GStreamer supports more then one or not,
                    // but it doesn't apply the SELECT flag to videos it appears. Therefore, assume
                    // the first track is the selected track.
                    let selected = video_track_index == 0;

                    if let Some(video_track) = video.get_video_track(video_track_index) {
                        let preview = video_track.preview();
                        preview.set_selected(selected);
                        preview.set_stream_id(stream.stream_id());
                    }

                    video_track_index += 1;
                },
                StreamType::AUDIO => {
                    let flags = stream.stream_flags();
                    let selected = flags.contains(gst::StreamFlags::SELECT);

                    if let Some(audio_track) = video.get_audio_track(audio_track_index) {
                        let preview = audio_track.preview();
                        preview.set_selected(selected);
                        preview.set_stream_id(stream.stream_id());
                    }

                    audio_track_index += 1;
                },
                StreamType::TEXT => {
                    let flags = stream.stream_flags();
                    let selected = flags.contains(gst::StreamFlags::SELECT);

                    if let Some(subtitle_track) = video.get_subtitle_track(subtitle_track_index) {
                        let preview = subtitle_track.preview();
                        preview.set_selected(selected);
                        preview.set_stream_id(stream.stream_id());
                    }

                    subtitle_track_index += 1;
                },
                _ => {
                    tracing::warn!(type=?stream.stream_type(), "unexpected stream type");
                }
            }
        }

        self.set_controls_enabled(true);
    }

    /// Callback when the end of the stream is reached.
    /// 
    /// When called, this will pause the video and will seek to the start of the video.
    fn video_ended(&self) {
        self.pause();
        self.video_seek(0);
    }

    /// Seek to a specific time within the stream.
    ///
    /// # Args
    ///
    /// `seconds`  The seconds from the start of the video to seek to. It is assumed that the value
    /// is not greater than the total length of the video.
    fn video_seek(&self, seconds: u64) {
        let playbin_element = self.imp().playbin_element
            .borrow()
            .as_ref()
            .unwrap()
            .clone();
        let value = gst::format::ClockTime::from_seconds(seconds);
        if let Err(error) = playbin_element.seek_simple(
            SeekFlags::FLUSH | SeekFlags::KEY_UNIT,
            value,
        ) {
            tracing::error!(?error, "failed to seek");
        }
    }
}

enum GstMessage {
    Eos,
    StreamCollection(StreamCollection)
}

fn enable_subtitles(playbin_element: &Element, enabled: bool) {
    let flags = playbin_element.property_value("flags");

    let flags_class = FlagsClass::with_type(flags.type_())
        .expect("flags_class was None");

    let flags = flags_class.builder_with_value(flags)
        .expect("flags builder was None")
        .set_by_nick("audio")
        .set_by_nick("video");

    let flags = if enabled {
        flags.set_by_nick("text")
    } else {
        flags.unset_by_nick("text")
    };

    let flags = flags
        .build()
        .unwrap();

    playbin_element.set_property_from_value("flags", &flags);
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

/// Generates a video filter sub-pipeline for ensuring the video respects the requested size of the
/// widget.
fn video_filter() -> Element {
    let video_scale_desc = String::from("videoscale add-borders=true");
    let caps_filter_desc = format!(
        "capsfilter caps=video/x-raw,width={},height={},pixel-aspect-ratio=1/1",
        VIDEO_FRAME_WIDTH,
        VIDEO_FRAME_HEIGHT
    );
    let video_filter_desc = format!("{} ! {}", video_scale_desc, caps_filter_desc);

    gst::parse::bin_from_description(&video_filter_desc, true)
        .unwrap()
        .upcast()
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::{Cell, RefCell};

    use gst::{Element, State};
    use gst::prelude::*;

    use gtk::{Box, ColumnView, Label, NoSelection, Scale, SingleSelection, StringList};

    use gtk::gio::ListStore;
    use gtk::glib::{self, Properties, SignalHandlerId};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::data::{AudioTrackObject, SubtitleTrackObject, VideoObject};
    use crate::ui::widget::DuelIconToggleButton;

    /// Implemenation for [`super::VideoPlayerWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::VideoPlayerWidget)]
    pub struct VideoPlayerWidget {
        /// The enabled/disabled status of the controls.
        #[property(name = "controls-enabled", get, set)]
        pub(super) controls_enabled: Cell<bool>,

        /// The active video.
        #[property(name = "video", get, set = Self::set_video, nullable)]
        pub(super) video: RefCell<Option<VideoObject>>,

        /// Provides an all-in-one abstraction for playing video/audio.
        ///
        /// It avoids the need to manually create the various audio/video elements while still
        /// providing the ability to control subtitles and the selected audio track.
        pub(super) playbin_element: RefCell<Option<Element>>,

        /// The button used to play/pause the video.
        pub(super) play_pause_button: RefCell<DuelIconToggleButton>,

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

        /// Signal identifiers for the selected video connections.
        pub(super) video_signals: RefCell<Vec<SignalHandlerId>>,

        /// Model for storing the list of available audio tracks.
        pub(super) audio_track_options: RefCell<StringList>,

        /// Model for storing the list of available subtitle tracks.
        pub(super) subtitle_track_options: RefCell<StringList>,
    }

    impl VideoPlayerWidget {
        /// Set the active video.
        /// 
        /// This is used as a custom setter for the video property so that when the video is set,
        /// a number of other actions (e.g. update track options) can be taken.
        ///
        /// # Args
        ///
        /// `video`   The new selected video.
        fn set_video(&self, video: Option<VideoObject>) {
            if let Some(old_video) = self.video.borrow().clone() {
                for signal_id in self.video_signals.borrow_mut().drain(..) {
                    old_video.disconnect(signal_id);
                }
            }

            if let Some(video) = &video {
                self.update_audio_track_options(video.audio_tracks());
                self.update_subtitle_track_options(video.subtitle_tracks());
            } else {
                self.update_audio_track_options(None);
                self.update_subtitle_track_options(None);
            }

            self.video.replace(video);
            self.obj().on_video_changed();
        }

        /// Updates the available audio tracks.
        ///
        /// # Args
        ///
        /// `audio_tracks`:  List store containing [`AudioTrackObject`] instances representing the
        /// available audio tracks for the selected video.
        fn update_audio_track_options(&self, audio_tracks: Option<ListStore>) {
            let names: Vec<String> = audio_tracks
                .map(|list| {
                    list.iter::<AudioTrackObject>()
                        .filter_map(Result::ok)
                        .map(|track| track.selector_display())
                        .collect()
                })
                .unwrap_or_default();

            let name_refs: Vec<&str> = names.iter()
                .map(String::as_str)
                .collect();

            let options = self.audio_track_options
                .borrow();
            options.splice(0, options.n_items(), &name_refs);
        }

        /// Updates the available subtitle tracks.
        ///
        /// # Args
        ///
        /// `subtitle_tracks`:  List store containing [`SubtitleTrackObject`] instances
        /// representing the available subtitle tracks for the selected video.
        fn update_subtitle_track_options(&self, subtitle_tracks: Option<ListStore>) {
            let names: Vec<String> = subtitle_tracks
                .map(|list| {
                    list.iter::<SubtitleTrackObject>()
                        .filter_map(Result::ok)
                        .map(|track| track.selector_display())
                        .collect()
                })
                .unwrap_or_default();

            let name_refs: Vec<&str> = names.iter()
                .map(String::as_str)
                .collect();

            let options = self.subtitle_track_options
                .borrow();
            options.splice(0, options.n_items(), &name_refs);
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
