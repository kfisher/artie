// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Screen for configuring the application.

use iced::{Alignment, Length};
use iced::widget::{Column, Row, Space};

use tracing::error;

use copy_srv::CopyService;

use crate::Message;
use crate::context::Context;
use crate::settings::ScaleFactor;
use crate::theme::Theme;
use crate::widget::Element;
use crate::widget::button::{Button, ButtonClass};
use crate::widget::container::{Container, ContainerClass};
use crate::widget::pick_list::{PickList, PickListClass};
use crate::widget::rule::Rule;
use crate::widget::text;
use crate::widget::text_input::{TextInput, TextInputClass};

/// Messages specific to the settings screen.
#[derive(Clone, Debug)]
pub enum SettingsScreenMessage {
    /// Opens the form for adding a new copy service.
    AddCopyService,

    /// Opens the dialog to confirm deletion of a copy service configuration.
    DeleteCopyService {
        index: usize,
    },

    /// Opens the form for editing an existing copy service.
    EditCopyService {
        index: usize
    },

    /// Discard pending changes to a copy service configuration.
    EditCopyServiceDiscard,

    /// The copy service configuration form's name field value has changed.
    EditCopyServiceNameInput {
        text: String,
    },

    /// The copy service configuration form's serial number field value has changed.
    EditCopyServiceSerialNumberInput {
        text: String,
    },
}

pub struct SettingsScreen {
    /// Form data for editing a copy service configuration.
    ///
    /// Will only be `Some` while a copy service is being edited. Only once service can be edited
    /// at a time.
    copy_service_form: Option<CopyServiceForm>,
}

impl SettingsScreen {
    /// Creates a new [`SettingsScreen`] instance.
    pub fn new() -> Self {
        Self {
            copy_service_form: None,
        }
    }

    /// Callback when a copy service's configuration has been applied.
    ///
    /// This will close the copy service form if open even if form is associated with a different
    /// copy service then the one that was updated. It will also close it even if this screen was
    /// not what initiated the change. The assumption is that this will only ever be called after
    /// the user hits Save. If support is ever added for editing the copy service settings
    /// elsewhere in the application, then it is assumed that will be on a different screen and 
    /// this function will not be called.
    pub fn copy_service_updated(&mut self) {
        self.copy_service_form = None;
    }

    /// Processes a settings screen message.
    pub fn process_message(&mut self, ctx: &Context, message: SettingsScreenMessage) {
        match message {
            SettingsScreenMessage::AddCopyService => {
                if self.copy_service_form.is_none() {
                    self.copy_service_form = Some(CopyServiceForm::new(ctx.copy_services.len()));
                } else {
                    error!("ignore AddCopyService - already editing");
                }
            },
            SettingsScreenMessage::DeleteCopyService { index: _ } => {
                // self.copy_services[index].display = CopyServiceDisplay::Delete;
            },
            SettingsScreenMessage::EditCopyService { index } => {
                let service = &ctx.copy_services[index];
                if self.copy_service_form.is_none() {
                    self.copy_service_form = Some(CopyServiceForm::from_service(index, service));
                } else {
                    error!("ignore EditCopyService - already editing");
                }
            },
            SettingsScreenMessage::EditCopyServiceDiscard => {
                if self.copy_service_form.is_some() {
                    self.copy_service_form = None;
                } else {
                    error!("ignore EditCopyServiceDiscard - not editing");
                }
            },
            SettingsScreenMessage::EditCopyServiceNameInput { text } => {
                if let Some(form) = self.copy_service_form.as_mut() {
                    form.input_name(&text, ctx);
                } else {
                    error!("ignore EditCopyServiceNameInput - not editing");
                }
            },
            SettingsScreenMessage::EditCopyServiceSerialNumberInput { text } => {
                if let Some(form) = self.copy_service_form.as_mut() {
                    form.input_serial_number(&text, ctx);
                } else {
                    error!("ignore EditCopyServiceSerialNumberInput - not editing");
                }
            },
        }
    }

    /// Generates the UI element for displaying the screen.
    pub fn view(&self, ctx: &Context) -> Element<'_> {
        let rows = Column::with_capacity(3)
            .push(text::heading1("Appearance"))
            .push(self.appearance_view(ctx))
            .push(text::heading1("Drives"))
            .push(self.copy_service_view(ctx))
            .spacing(16);

        Container::new(rows)
            .class(ContainerClass::Default)
            .align_x(Alignment::Center)
            .padding([16, 32])
            .into()
    }

    /// Generates the UI element for displaying the section for the appearance settings.
    fn appearance_view(&self, ctx: &Context) -> Element<'_> {
        fn form_row<'a, T>(label: &'a str, control: T) -> Row<'a, crate::Message, Theme> 
        where 
            T: Into<Element<'a>> + 'a
        {
            Row::with_capacity(2)
                .push(text::label(label).width(Length::Fill))
                .push(control)
                .align_y(Alignment::Center)
                .spacing(8)
                .padding(16)
                .width(Length::Fill)
        }

        let scale_factor = form_row(
            "Scale Factor",
            PickList::new(
                ScaleFactor::OPTIONS,
                Some(ctx.settings.general.scale_factor),
                Message::SetScaleFactor,
            )
            .width(100),
        );

        let theme = form_row(
            "Theme",
            PickList::new(
                Theme::ALL,
                Some(ctx.settings.general.theme),
                Message::SetTheme,
            )
            .width(100),
        );

        let content = Column::with_capacity(3)
            .push(scale_factor)
            .push(Rule::horizontal(1))
            .push(theme);

        Container::new(content)
            .class(ContainerClass::Panel)
            .max_width(1080)
            .into()
    }

    /// Generates the UI element for displaying copy service information.
    fn copy_service_item_view(&self, index: usize, copy_service: &CopyService) -> Element<'_> {
        let mut cols: Vec<Element<'_>> = Vec::with_capacity(3);

        let name = Column::with_capacity(2)
            .push(text::label(copy_service.name.clone()))
            .push(text::small_subtext("Name"))
            .width(Length::Fill);
        cols.push(name.into());

        let serial_number = Column::with_capacity(2)
            .push(text::label(copy_service.drive.serial_number.clone()))
            .push(text::small_subtext("Serial Number"))
            .width(Length::Fill);
        cols.push(serial_number.into());

        // Only show the edit controls if not currently editing a copy service configuration.
        //
        // TODO: Will also need to ensure that the copy service is not running a copy operation
        //       since editing a copy service mid operation could be bad. 
        if self.copy_service_form.is_none() {
            let edit_button = Button::new(ButtonClass::Default)
                .icon("fontawesome.v7.solid.edit")
                .icon_size(20.0)
                .on_press(
                    Message::SettingsScreen(
                        SettingsScreenMessage::EditCopyService { index }
                    )
                );
            cols.push(edit_button.into());
        }

        Row::with_children(cols)
            .align_y(Alignment::Center)
            .padding(16)
            .width(Length::Fill)
            .spacing(8)
            .into()
    }

    /// Generates the UI element for displaying the section for the copy service settings.
    fn copy_service_view(&self, ctx: &Context) -> Element<'_> {
        let mut rows: Vec<Element<'_>> = Vec::with_capacity(ctx.copy_services.len() * 2);

        for (index, service) in ctx.copy_services.iter().enumerate() {
            if index > 0 {
                rows.push(Rule::horizontal(1).into())
            }

            let row = match &self.copy_service_form {
                Some(form) => if form.index == index {
                    form.view()
                } else {
                    self.copy_service_item_view(index, &service)
                },
                None => self.copy_service_item_view(index, &service),
            };

            rows.push(row);
        }

        // Account for a new service being added.
        if let Some(form) = &self.copy_service_form && form.index >= ctx.copy_services.len() {
            if ctx.copy_services.len() > 0 {
                rows.push(Rule::horizontal(1).into())
            }

            rows.push(form.view());
        }

        let content = Container::new(Column::with_children(rows))
            .class(ContainerClass::Panel);

        Column::with_capacity(2)
            .push(content)
            .push(Button::new(ButtonClass::Primary)
                .icon("fontawesome.v7.solid.plus")
                .label("Add")
                .on_press(Message::SettingsScreen(SettingsScreenMessage::AddCopyService)))
            .max_width(1080)
            .spacing(16)
            .into()
    }
}

/// Data related to editing a copy service's configuration.
pub struct CopyServiceForm {
    /// Index of the copy service within the application context.
    ///
    /// It is expected that the copy service will not change its order while its being edited.
    index: usize,

    /// Name of the service.
    ///
    /// This value will be updated whenever the user inputs text into the name field. See
    /// [`CopyServiceForm::input_name`].
    name: String,

    /// The serial number of the optical drive associated with the service.
    ///
    /// This value will be updated whenever the user inputs text into the name field. See
    /// [`CopyServiceForm::input_serial_number`].
    serial_number: String,

    /// Indicates if the changes can be applied.
    ///
    /// This will enable or disable the save button. The button will be disabled if the form's data
    /// is invalid or if none of the values have changed.
    can_apply: bool,
}

impl CopyServiceForm {
    /// Creates a [`CopyServiceForm`] instance for adding a new service.
    pub fn new(index: usize) -> Self {
        Self {
            index,
            name: String::from(""),
            serial_number: String::from(""),
            can_apply: false,
        }
    }

    /// Creates a [`CopyServiceForm`] instance for editing an existing service.
    pub fn from_service(index: usize, service: &CopyService) -> Self {
        Self {
            index,
            name: service.name.clone(),
            serial_number: service.drive.serial_number.clone(),
            can_apply: false,
        }
    }

    /// Callback when the name field changes.
    pub fn input_name(&mut self, text: &str, ctx: &Context) {
        self.name = text.to_owned();
        self.validate(ctx);
    }

    /// Callback when the serial number changes.
    pub fn input_serial_number(&mut self, text: &str, ctx: &Context) {
        self.serial_number = text.to_owned();
        self.validate(ctx);
    }

    /// Generates the UI element for displaying the form.
    fn view(&self) -> Element<'_> {
        let input = TextInput::new("Name", &self.name)
            .on_input(move |text| {
                Message::SettingsScreen(
                    SettingsScreenMessage::EditCopyServiceNameInput { 
                        text 
                    }
                )
            });

        let label = text::small_subtext("Name");

        let name = Column::with_capacity(2)
            .push(input)
            .push(label)
            .width(Length::Fill);

        let input = TextInput::new("Serial Number", &self.serial_number)
            .on_input(move |text| {
                Message::SettingsScreen(
                    SettingsScreenMessage::EditCopyServiceSerialNumberInput { 
                        text 
                    }
                )
            });

        let label = text::small_subtext("Serial Number");

        let serial_number = Column::with_capacity(2)
            .push(input)
            .push(label)
            .width(Length::Fill);

        let form = Row::with_capacity(2)
            .push(name)
            .push(serial_number)
            .spacing(8);

        let apply_message = if self.can_apply {
            let message = Message::UpdateCopyService { 
                index: self.index,
                name: self.name.clone(), 
                serial_number: self.serial_number.clone(),
            };
            Some(message)
        } else {
            None
        };

        let apply_button = Button::new(ButtonClass::Success)
            .icon("fontawesome.v7.solid.check")
            .label("Save")
            .on_press_maybe(apply_message);

        let discard_button = Button::new(ButtonClass::Default)
            .icon("fontawesome.v7.solid.cancel")
            .label("Cancel")
            .on_press(Message::SettingsScreen( SettingsScreenMessage::EditCopyServiceDiscard));

        let delete_button = Button::new(ButtonClass::Danger)
            .icon("fontawesome.v7.solid.trash")
            .label("Delete")
            .on_press(
                Message::SettingsScreen(
                    SettingsScreenMessage::DeleteCopyService { index: self.index }
                )
            );

        let controls = Row::with_capacity(2)
            .push(apply_button)
            .push(discard_button)
            .push(Space::with_width(Length::Fill))
            .push(delete_button)
            .spacing(8);

        Column::with_capacity(2)
            .push(form)
            .push(controls)
            .padding(16)
            .width(Length::Fill)
            .spacing(8)
            .into()
    }

    /// Validates the current values.
    ///
    /// This will update the `can_apply` field based on the validity of the values including if the
    /// values have changed or not.
    fn validate(&mut self, ctx: &Context) {
        // Both name and serial_number must contain at least one non-whitespace character.
        if self.name.trim().is_empty() || self.serial_number.trim().is_empty() {
            self.can_apply = false;
            return;
        }

        // The drive serial number must be unique.
        //
        // While the name should also be unique from a usability standpoint, there isn't a
        // programmatic reason to do so. Therefore, we won't enforce uniqueness on the off chance
        // a user wants duplicate names for some reason.
        for (index, service) in ctx.copy_services.iter().enumerate() {
            // Skip the service being edited.
            if index == self.index {
                // Disable the button if none of the values have changed.
                if self.name == service.name && self.serial_number == service.drive.serial_number {
                    self.can_apply = false;
                    return;
                }
                continue;
            }

            if self.serial_number.eq_ignore_ascii_case(&service.drive.serial_number) {
                self.can_apply = false;
                return;
            }
        }

        self.can_apply = true;
    }
}

// TESTING TODO:
// - Validation
// - Process Message
