// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

mod drive;
mod form;

use std::borrow::Cow;

use iced::advanced::Widget;
use iced::{Alignment, Border, Length};
use iced::font::{Family, Font, Style as FontStyle, Weight};
use iced::widget::{Column, Row, Space};
use iced::widget::container::Style as ContainerStyle;

use tracing::error;

use copy_srv::{CopyService, State};

use optical_drive::DiscState;

use crate::Message;
use crate::context::Context;
use crate::theme::Theme;
use crate::theme::palette::{ColorSet, Palette};
use crate::widget::Element;
use crate::widget::button::{Button, ButtonClass};
use crate::widget::container::{Container, ContainerClass};
use crate::widget::rule::{Rule, RuleClass};
use crate::widget::text::{Text, TextClass};
use crate::widget::icon::{self, IconClass};

use drive::DriveComponent;
use form::{CopyForm, MediaType};

/// Messages specific to the settings screen.
#[derive(Clone, Debug)]
pub enum CopyScreenMessage {
    /// Clears the copy operation form.
    ClearCopyForm {
        index: usize,
    },

    /// Updates the disc number field in the copy operation form.
    UpdateCopyFormDiscNumber {
        index: usize,
        text: String,
    },

    /// Updates the location field in the copy operation form.
    UpdateCopyFormLocation {
        index: usize,
        text: String,
    },

    /// Updates the media type field in the copy operation form.
    UpdateCopyFormMediaType {
        index: usize,
        media_type: MediaType,
    },

    /// Updates the memo field in the copy operation form.
    UpdateCopyFormMemo {
        index: usize,
        text: String,
    },

    /// Updates the release year field in the copy operation form.
    UpdateCopyFormReleaseYear {
        index: usize,
        text: String,
    },

    /// Updates the season number field in the copy operation form.
    UpdateCopyFormSeasonNumber {
        index: usize,
        text: String,
    },

    /// Updates the title field in the copy operation form.
    UpdateCopyFormTitle {
        index: usize,
        text: String,
    },
}

impl From<CopyScreenMessage> for Message {
    fn from(value: CopyScreenMessage) -> Self {
        Message::CopyScreen(value)
    }
}

/// Screen for copying titles from DVDs and Blu-rays.
#[derive(Default)]
pub struct CopyScreen {
    drive_components: Vec<DriveComponent>,
}

impl CopyScreen {
    /// Create a new [`CopyScreen`] instance.
    pub fn new(ctx: &Context) -> CopyScreen {
        CopyScreen { 
            drive_components: create_drive_widgets(ctx),
        }
    }

    /// Callback when a copy service's configuration changes so that the screen can update any
    /// internal data effected.
    pub fn copy_service_updated(&mut self, ctx: &Context) {
        // TODO: Currently we'll just recreate the internal data. In the future, this will need to 
        //       be smarter once there is data that should not be cleared.
        self.drive_components = create_drive_widgets(ctx);
    }

    /// Processes a copy screen message.
    pub fn process_message(&mut self, _ctx: &Context, message: CopyScreenMessage) {
        match message {
            CopyScreenMessage::ClearCopyForm { index } => {
                self.process_form_update(index, |form| form.clear());
            },
            CopyScreenMessage::UpdateCopyFormDiscNumber { index, text } => {
                self.process_form_update(index, |form| form.input_disc_number(&text));
            },
            CopyScreenMessage::UpdateCopyFormLocation { index, text } => {
                self.process_form_update(index, |form| form.input_location(&text));
            },
            CopyScreenMessage::UpdateCopyFormMediaType { index, media_type } => {
                self.process_form_update(index, |form| form.input_media_type(media_type));
            },
            CopyScreenMessage::UpdateCopyFormMemo { index, text } => {
                self.process_form_update(index, |form| form.input_memo(&text));
            },
            CopyScreenMessage::UpdateCopyFormReleaseYear { index, text } => {
                self.process_form_update(index, |form| form.input_release_year(&text));
            },
            CopyScreenMessage::UpdateCopyFormSeasonNumber { index, text } => {
                self.process_form_update(index, |form| form.input_season_number(&text));
            },
            CopyScreenMessage::UpdateCopyFormTitle { index, text } => {
                self.process_form_update(index, |form| form.input_title(&text));
            },
        }
    }

    /// Ticks the screen (used for animations).
    pub fn tick(&mut self, delta_time: f32) {
        for drive_component in self.drive_components.iter_mut() {
            drive_component.tick(delta_time);
        }
    }

    /// Indicates if this screen needs to be ticked.
    pub fn should_tick(&self) -> bool {
        self.drive_components.iter().any(|drive_component| drive_component.should_tick())
    }

    /// Generates the view for the screen.
    pub fn view<'a>(&'a self, ctx: &'a Context) -> Element<'a> {
        let widgets: Vec<Element<'_>> = self.drive_components.iter().zip(ctx.copy_services.iter())
            .map(|(component, service)| component.view(service))
            .collect();

        Column::with_children(widgets)
            .align_x(Alignment::Center)
            .spacing(16)
            .padding([18, 36])
            .into()
    }

    /// Processes a form update for the copy service at the provided index using the provided
    /// processing function.
    fn process_form_update<T>(&mut self, index: usize, process: T)
        where 
            T: Fn(&mut CopyForm)
    {
        if let Some(widget) = self.drive_components.get_mut(index) {
            process(&mut widget.form);
            widget.form.validate();
        } else {
            error!(index=index, "cannot process form update, index was out of range");
        }
    }
}

/// Generates the list of drive widgets from the provided context.
fn create_drive_widgets(ctx: &Context) -> Vec<DriveComponent> {
    ctx.copy_services.iter().enumerate()
        .map(|(index, service)| DriveComponent::from_service(index, service))
        .collect()
}

