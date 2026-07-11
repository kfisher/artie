// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::RefCell;

use gio::ListStore;
use glib::{self, Object, Properties};
use gtk::{
    Align,
    Box,
    ColumnView,
    ColumnViewColumn,
    Label,
    ListItem,
    NoSelection,
    Orientation,
    SignalListItemFactory,
    StringList,
};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use handbrake;

use crate::ui::data::{AudioTrackObject, AudioEncodeOptionObject, VideoObject};
use crate::ui::widget::{EntryWidget, DropDownWidget, IconButton};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::AudioTrackFieldWidget)]
pub struct AudioTrackFieldWidget {
    /// The active video.
    #[property(get, set = Self::set_video, nullable)]
    pub(super) video: RefCell<Option<VideoObject>>,

    /// The list of audio tracks that will be encoded when the video is transcoded.
    pub(super) encode_list: RefCell<Option<ListStore>>,

    /// Dropdown for selecting the encoding method to use.
    pub(super) encoder_dropdown: RefCell<Option<DropDownWidget>>,

    /// List of available encoders.
    pub(super) encoder_list: RefCell<Option<StringList>>,

    /// Button used to add tracks to the encode list.
    pub(super) add_button: RefCell<Option<IconButton>>,

    /// The entry used for editing the track name when adding a new track to the list.
    pub(super) name_entry: RefCell<Option<EntryWidget>>,

    /// Dropdown for selecting the HandBrake preset to use when transcoding.
    pub(super) source_track_dropdown: RefCell<Option<DropDownWidget>>,
}

// TODO: Adding should have validation.
// TODO: Should add controls to disable adding new tracks after a certain count is reached.

impl AudioTrackFieldWidget {
    /// Validates the track input controls and enables or disables the Add button accordingly.
    fn add_track_input_changed(&self) {
        let Some(add_button) = self.add_button.borrow().clone() else {
            return;
        };

        add_button.set_sensitive(self.can_add_track());
    }

    /// Callback when the Add button is clicked.
    fn add_clicked(&self) {
        let encode_list = self.encode_list
            .borrow()
            .clone()
            .unwrap();

        let number = encode_list.n_items() + 1;

        let source_track_index = self.source_track_dropdown
            .borrow()
            .as_ref()
            .unwrap()
            .selected();

        let source_track = self.video
            .borrow()
            .as_ref()
            .unwrap()
            .get_audio_track(source_track_index)
            .unwrap();

        let encoder_index = self.encoder_dropdown
            .borrow()
            .as_ref()
            .unwrap()
            .selected();

        let encoder = self.encoder_list
            .borrow()
            .as_ref()
            .unwrap()
            .string(encoder_index)
            .unwrap();

        let name = self.name_entry
            .borrow()
            .as_ref()
            .unwrap()
            .text();

        let item = AudioEncodeOptionObject::new(
            number as u8,
            &source_track,
            &encoder,
            &name,
        );
        encode_list.append(&item);

        self.add_track_input_changed();
    }

    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_orientation(Orientation::Vertical);
        obj.set_spacing(4);
        obj.set_vexpand(true);
        obj.append(&self.create_controls());
        obj.append(&self.create_track_table());
    }

    /// Returns `true` if the track inputs are valid and the encode list can accept additional
    /// tracks or `false` otherwise.
    fn can_add_track(&self) -> bool {
        let Some(source_track_dropdown) = self.source_track_dropdown.borrow().clone() else {
            return false;
        };

        let source_track_index = source_track_dropdown.selected();
        if source_track_index == gtk::INVALID_LIST_POSITION {
            return false;
        }

        let Some(video) = self.video.borrow().clone() else {
            return false;
        };

        if video.get_audio_track(source_track_index).is_none() {
            return false;
        };

        let Some(encoder_dropdown) = self.encoder_dropdown.borrow().clone() else {
            return false;
        };

        let encoder_index = encoder_dropdown.selected();
        if encoder_index >= gtk::INVALID_LIST_POSITION {
            return false;
        }

        let Some(encoder_list) = self.encoder_list.borrow().clone() else {
            return false;
        };

        if encoder_list.string(encoder_index).is_none() {
            return false;
        }

        let Some(name_entry) = self.name_entry.borrow().clone() else {
            return false;
        };

        // TODO: Entry should have built-in validation.
        if name_entry.text().is_empty() {
            return false;
        }

        let Some(encode_list) = self.encode_list.borrow().clone() else {
            return false;
        };

        // TODO: Use a constant
        if encode_list.n_items() >= 5 {
            return false;
        }

        true
    }


    /// Creates the controls for adding tracks to the track table.
    fn create_controls(&self) -> Box {
        let controls = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        let source_track_dropdown = DropDownWidget::builder()
            .label("Source Track")
            .build();
        controls.append(&source_track_dropdown);

        let this = self;
        source_track_dropdown.connect_select_notify(glib::clone!(
            #[weak]
            this,
            move |_| {
                this.source_track_changed();
            }
        ));

        let encoder_dropdown = DropDownWidget::builder()
            .label("Encoder")
            .options(handbrake::audio_encoders())
            .build();
        controls.append(&encoder_dropdown);

        let this = self;
        encoder_dropdown.connect_select_notify(glib::clone!(
            #[weak]
            this,
            move |_| {
                this.add_track_input_changed();
            }
        ));

        let name_entry = EntryWidget::builder()
            .label("Name")
            .build();
        controls.append(&name_entry);

        let this = self;
        name_entry.connect_text_notify(glib::clone!(
            #[weak]
            this,
            move |_| {
                this.add_track_input_changed();
            }
        ));

        let add_button = IconButton::builder()
            .icon_name("fontawesome.v7.solid.plus")
            .label("Add")
            .secondary_button()
            .build();
        add_button.set_sensitive(false);
        controls.append(&add_button);

        let this = self;
        add_button.connect_clicked(glib::clone!(
            #[weak]
            this,
            move |_| {
                this.add_clicked();
            }
        ));

        self.add_button.replace(Some(add_button));
        self.encoder_list.replace(Some(encoder_dropdown.model()));
        self.encoder_dropdown.replace(Some(encoder_dropdown));
        self.name_entry.replace(Some(name_entry));
        self.source_track_dropdown.replace(Some(source_track_dropdown));

        controls
    }

    /// Creates the table for displaying the tracks that will be part of the transcode.
    fn create_track_table(&self) -> ColumnView {
        let encode_list = ListStore::new::<AudioEncodeOptionObject>();
        let selection_model = NoSelection::new(Some(encode_list.clone()));

        let column_view = ColumnView::builder()
            .hexpand(true)
            .vexpand(true)
            .model(&selection_model)
            .build();

        let number_factory = SignalListItemFactory::new();
        number_factory.connect_setup(label_column_setup);
        number_factory.connect_bind(|_factor, obj| {
            let item = obj
                .downcast_ref::<ListItem>()
                .unwrap();
            let label = item.child()
              .unwrap()
              .downcast::<Label>()
              .unwrap();
            let encode_option = item.item()
                .unwrap()
                .downcast::<AudioEncodeOptionObject>()
                .unwrap();
            encode_option.bind_property("track-number", &label, "label")
                .transform_to(|_binding, track_number: u8| {
                    Some(track_number.to_string())
                })
                .sync_create()
                .build();
        });

        let number_column = ColumnViewColumn::builder()
            .title("No.")
            .factory(&number_factory)
            .build();
        column_view.append_column(&number_column);

        let source_track_factory = SignalListItemFactory::new();
        source_track_factory.connect_setup(label_column_setup);
        source_track_factory.connect_bind(|_factor, obj| {
            let item = obj
                .downcast_ref::<ListItem>()
                .unwrap();
            let label = item.child()
              .unwrap()
              .downcast::<Label>()
              .unwrap();
            let encode_option = item.item()
                .unwrap()
                .downcast::<AudioEncodeOptionObject>()
                .unwrap();
            encode_option.bind_property("source-track", &label, "label")
                .transform_to(|_binding, track: AudioTrackObject| {
                    Some(track.selector_display())
                })
                .sync_create()
                .build();
        });

        let source_track_column = ColumnViewColumn::builder()
            .title("Source Track")
            .factory(&source_track_factory)
            .build();
        column_view.append_column(&source_track_column);

        let encoder_factory = SignalListItemFactory::new();
        encoder_factory.connect_setup(label_column_setup);
        encoder_factory.connect_bind(|_factory, obj| {
            let item = obj
                .downcast_ref::<ListItem>()
                .unwrap();
            let label = item.child()
              .unwrap()
              .downcast::<Label>()
              .unwrap();
            let encode_option = item.item()
                .unwrap()
                .downcast::<AudioEncodeOptionObject>()
                .unwrap();
            encode_option.bind_property("encoder", &label, "label")
                .sync_create()
                .build();
        });

        let encoder_column = ColumnViewColumn::builder()
            .title("Encoder")
            .factory(&encoder_factory)
            .build();
        column_view.append_column(&encoder_column);

        let name_factory = SignalListItemFactory::new();
        name_factory.connect_setup(label_column_setup);
        name_factory.connect_bind(|_factor, obj| {
            let item = obj
                .downcast_ref::<ListItem>()
                .unwrap();
            let label = item.child()
              .unwrap()
              .downcast::<Label>()
              .unwrap();
            let encode_option = item.item()
                .unwrap()
                .downcast::<AudioEncodeOptionObject>()
                .unwrap();
            encode_option.bind_property("name", &label, "label")
                .sync_create()
                .build();
        });

        let name_column = ColumnViewColumn::builder()
            .expand(true)
            .title("Name")
            .factory(&name_factory)
            .build();
        column_view.append_column(&name_column);

        let controls_factory = SignalListItemFactory::new();
        let this = self;
        controls_factory.connect_setup(glib::clone!(
            #[weak]
            this,
            move |_factory, obj| {
                let controls = Box::builder()
                    .orientation(Orientation::Horizontal)
                    .build();
                let delete_button = IconButton::builder()
                    .icon_name("fontawesome.v7.solid.trash-symbolic")
                    .danger_button()
                    .ghost_button()
                    .build();
                delete_button.connect_clicked(glib::clone!(
                    #[weak]
                    this,
                    #[weak]
                    obj,
                    move |_| {
                        let encode_option = obj.downcast_ref::<ListItem>()
                            .unwrap()
                            .item()
                            .unwrap()
                            .downcast::<AudioEncodeOptionObject>()
                            .unwrap();
                        this.remove_encode_option(&encode_option);
                    }
                ));
                controls.append(&delete_button);
                obj.downcast_ref::<ListItem>()
                    .unwrap()
                    .set_child(Some(&controls));
            }
        ));
        let controls_column = ColumnViewColumn::builder()
            .title("")
            .factory(&controls_factory)
            .build();
        column_view.append_column(&controls_column);

        self.encode_list.replace(Some(encode_list));

        column_view
    }

    /// Removes an item from the encode list.
    fn remove_encode_option(&self, encode_option: &AudioEncodeOptionObject) {
        let encode_list = self.encode_list
            .borrow()
            .clone()
            .unwrap();

        let index = encode_option.track_number() as u32 - 1;
        encode_list.remove(index);

        for (index, encode_option) in encode_list.iter::<AudioEncodeOptionObject>().enumerate() {
            if let Ok(encode_option) = encode_option {
                let number = (index + 1) as u8;
                encode_option.set_track_number(number);
            }
        }

        self.add_track_input_changed();
    }

    /// Resets the form fields back to default values and the selected video to `None`.
    fn reset(&self) {
        self.source_track_dropdown
            .borrow()
            .as_ref()
            .unwrap()
            .set_model(&StringList::default());
        self.video.replace(None);
    }

    /// Setter for the active video.
    ///
    /// # Args
    ///
    /// `video`:  The newly selected video. If `Some`, the form will be updated to reflect the
    /// provided video. If `None`, the form's values will be reset back to default. In both cases,
    /// any user provided changes will be reset.
    fn set_video(&self, video: Option<VideoObject>) {
        let Some(video) = video else {
            self.reset();
            return;
        };

        let Some(audio_tracks) = video.audio_tracks() else {
            self.reset();
            return;
        };

        // Replacement must occur here due to the following triggering events that expect to have
        // the video set.
        self.video.replace(Some(video));

        let audio_tracks = audio_tracks.iter::<AudioTrackObject>()
            .filter_map(Result::ok)
            .map(|track| track.selector_display());
        self.source_track_dropdown
            .borrow()
            .as_ref()
            .unwrap()
            .set_model(&StringList::from_iter(audio_tracks));
    }

    /// Callback when the selected source track changes.
    fn source_track_changed(&self) {
        let video = self.video
            .borrow()
            .clone();
        let Some(video) = video else {
            return;
        };

        let selected = self.source_track_dropdown
            .borrow()
            .as_ref()
            .unwrap()
            .selected();

        let text = match video.get_audio_track(selected) {
            Some(audio_track) => audio_track.name(),
            None => String::default(),
        };

        self.name_entry
            .borrow()
            .as_ref()
            .unwrap()
            .set_text(&text);

        self.add_track_input_changed();
    }
}

#[glib::object_subclass]
impl ObjectSubclass for AudioTrackFieldWidget {
    const NAME: &'static str = "ArtieAudioTrackFieldWidget";
    type Type = super::AudioTrackFieldWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for AudioTrackFieldWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for AudioTrackFieldWidget {
}

impl BoxImpl for AudioTrackFieldWidget {
}

/// Function used as the callback when setting up a column in a column view when the column
/// contains a label.
///
/// # Args
///
/// `factory`:  The factory instance. 
///
/// `obj`:  The [`ListItem`] object associated with the column.
fn label_column_setup(_factory: &SignalListItemFactory, obj: &Object) {
    let label = Label::builder()
        .halign(Align::Start)
        .hexpand(true)
        .build();
    obj.downcast_ref::<ListItem>()
        .unwrap()
        .set_child(Some(&label));
}
