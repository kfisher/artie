// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result};

use iced::font::{Font, Weight};
use iced::widget::{Column, Row};

use model::MediaType;

use crate::Message;
use crate::theme::Theme;
use crate::widget::Element;
use crate::widget::pick_list::PickList;
use crate::widget::text::{Text, TextClass};
use crate::widget::text_input::{TextInput, TextInputClass};

use super::CopyScreenMessage;

/// Inputs for initiating a copy requests.
#[derive(Default)]
pub struct CopyForm {
    /// The type of media being copied (Movie or TV Show).
    media_type: MediaType,

    /// The title of the show or movie.
    title: Field<String>,

    /// The release year of the movie or show (first season premier).
    release_year: Field<String>,

    /// The season of the show the title belongs to.
    ///
    /// This is only required for television shows. It will be ignored for movies.
    season_number: Field<String>,

    /// Disc number.
    disc_number: Field<String>,

    /// Location where the disc is stored.
    location: Field<String>,

    /// Additional information provided by the user.
    memo: Field<String>,

    /// Indicates if the copy parameters are valid.
    valid: bool,
}

impl CopyForm {
    /// Creates a [`CopyForm`] instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the form values returning it to the default values.
    pub fn clear(&mut self) {
        self.media_type = MediaType::Movie;
        self.title.reset();
        self.release_year.reset();
        self.season_number.reset();
        self.disc_number.reset();
        self.location.reset();
        self.memo.reset();
        self.valid = false;
    }

    /// Callback when the disc number changes.
    pub fn input_disc_number(&mut self, text: &str) {
        if text.len() > 2 {
            return 
        }

        if text.is_empty() || text.parse::<u16>().is_ok() {
            self.disc_number.value = text.to_owned();
            self.disc_number.enable_errors = true;
        }
    }

    /// Callback when the location changes.
    pub fn input_location(&mut self, text: &str) {
        self.location.value = text.to_owned();
        self.location.enable_errors = true;
    }

    /// Callback when the media_type changes.
    pub fn input_media_type(&mut self, media_type: MediaType) {
        self.media_type = media_type;
    }

    /// Callback when the memo changes.
    pub fn input_memo(&mut self, text: &str) {
        self.memo.value = text.to_owned();
    }

    /// Callback when the release year changes.
    pub fn input_release_year(&mut self, text: &str) {
        if text.len() > 4 {
            return 
        }

        if text.is_empty() || text.parse::<u16>().is_ok() {
            self.release_year.value = text.to_owned();
            self.release_year.enable_errors = true;
        }
    }

    /// Callback when the season number changes.
    pub fn input_season_number(&mut self, text: &str) {
        if text.len() > 2 {
            return 
        }

        if text.is_empty() || text.parse::<u16>().is_ok() {
            self.season_number.value = text.to_owned();
            self.season_number.enable_errors = true;
        }
    }

    /// Callback when the title changes.
    pub fn input_title(&mut self, text: &str) {
        self.title.value = text.to_owned();
        self.title.enable_errors = true;
    }

    /// Returns `true` if the form's data is valid, `false` otherwise.
    pub fn valid(&self) -> bool {
        self.valid 
    }

    /// Validates the form based on the current values.
    pub fn validate(&mut self) {
        self.title.validate(|value| !value.trim().is_empty());

        self.location.validate(|value| !value.trim().is_empty());

        self.release_year.validate(|value| {
            if let Ok(year) = value.parse::<u16>() && (1000..=9999).contains(&year) {
                return true;
            }

            false
        });

        self.disc_number.validate(|value| {
            if let Ok(disc_number) = value.parse::<u16>() && disc_number > 0 {
                return true;
            }

            false
        });

        if self.media_type == MediaType::Show {
            self.season_number.validate(|value| {
                if let Ok(season_number) = value.parse::<u16>() && season_number > 0 {
                    return true;
                }

                false
            });
        } else {
            self.season_number.valid = true;
        }

        self.valid = self.title.valid 
            && self.location.valid
            && self.release_year.valid
            && self.disc_number.valid
            && self.season_number.valid;
    }

    /// Generates the UI element for the form.
    pub fn view(&self, index: usize) -> Element<'_> {
        let mut row_0: Vec<Element<'_>> = Vec::with_capacity(5);

        let media_type = PickList::new(
            MediaType::ALL,
            Some(self.media_type),
            move |media_type| Message::CopyScreen(
                CopyScreenMessage::UpdateCopyFormMediaType { index, media_type }
            ),
        );
        let media_type = form_field(media_type, " Type");
        row_0.push(media_type.into());

        let title = form_text_input(
            " Title",
            &self.title, 
            move |text| CopyScreenMessage::UpdateCopyFormTitle { index, text }
        );
        row_0.push(title.into());

        let release_year = form_text_input(
            " Release Year",
            &self.release_year,
            move |text| CopyScreenMessage::UpdateCopyFormReleaseYear { index, text }
        ).width(100);
        row_0.push(release_year.into());

        let mut row_1: Vec<Element<'_>> = Vec::with_capacity(5);

        let disc_number = form_text_input(
            " Disc Number",
            &self.disc_number,
            move |text| CopyScreenMessage::UpdateCopyFormDiscNumber { index, text }
        ).width(100);
        row_1.push(disc_number.into());

        if self.media_type == MediaType::Show {
            let season_number = form_text_input(
                " Season",
                &self.season_number,
                move |text| CopyScreenMessage::UpdateCopyFormSeasonNumber { index, text }
            ).width(100);
            row_1.push(season_number.into());
        }

        let location = form_text_input(
            " Storage Location",
            &self.location,
            move |text| CopyScreenMessage::UpdateCopyFormLocation { index, text }
        );
        row_1.push(location.into());

        let memo = form_text_input(
            " Memo",
            &self.memo,
            move |text| CopyScreenMessage::UpdateCopyFormMemo { index, text }
        );
        row_1.push(memo.into());

        let row_0 = Row::with_children(row_0)
            .spacing(8);

        let row_1 = Row::with_children(row_1)
            .spacing(8);

        Column::with_capacity(2)
            .push(row_0)
            .push(row_1)
            .padding(8)
            .spacing(8)
            .into()
    }
}

/// Represents a field in a form.
#[derive(Default)]
struct Field<T> 
where 
    T: Default
{
    /// The value of the field.
    pub value: T,

    /// Indicates if the field is valid.
    pub valid: bool,

    /// Indicates if the form control should indicate an error if invalid.
    /// 
    /// This is used to prevent errors indications from showing when the form is first initially 
    /// displayed.
    pub enable_errors: bool,
}

impl<T> Field<T> 
where 
    T: Default 
{
    /// Resets the field back to default values.
    pub fn reset(&mut self) {
        self.value = T::default();
        self.valid = false;
        self.enable_errors = false;
    }

    /// Helper for validating the field.
    pub fn validate(&mut self, f: fn(&T) -> bool) {
        self.valid = f(&self.value);
    }
}

/// Generates the UI element for a form field.
fn form_field<'a, T>(input: T, label: &'a str) -> Column<'a, Message, Theme>
where 
    T: Into<Element<'a>>
{
    Column::with_capacity(2)
        .push(input)
        .push(form_label(label))
}

/// Creates a text widget for the form labels.
fn form_label<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .class(TextClass::Subtext)
        .size(12)
        .font(Font {
            weight: Weight::Normal,
            ..Font::default()
        })
}

/// Generates the UI element for a text input form field.
fn form_text_input<'a, T>(
    label: &'a str,
    field: &Field<String>,
    message: T
) -> Column<'a, Message, Theme> 
where 
    T: Fn(String) -> CopyScreenMessage + 'a
{
    let class = match field.valid || !field.enable_errors {
        true => TextInputClass::Default,
        false => TextInputClass::Invalid,
    };

    let input = TextInput::new("", &field.value)
        .class(class)
        .on_input(move |text| Message::CopyScreen(message(text)));
    form_field(input, label)
}

// TODO: Validation Test
