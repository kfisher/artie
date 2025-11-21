// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! [`crate::Screen::Settings`] screen.

use std::path::{Path, PathBuf};

use iced::{Alignment, Length, Task};
use iced::widget::{Column, Row, Space};

use rfd::{AsyncFileDialog, FileDialog};

use tracing::error;

use copy_srv::CopyService;

use crate::Message;
use crate::context::Context;
use crate::settings::ScaleFactor;
use crate::theme::Theme;
use crate::widget::Element;
use crate::widget::button::{Button, ButtonClass};
use crate::widget::container::{Container, ContainerClass};
use crate::widget::dialog::ConfirmDeleteDialog;
use crate::widget::pick_list::PickList;
use crate::widget::rule::{Rule, RuleClass};
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

    /// Open the file dialog for selecting the archive directory.
    OpenArchiveFileDialog,

    /// Open the file dialog for selecting the data directory.
    OpenDataFileDialog,

    /// Open the file dialog for selecting the inbox directory.
    OpenInboxFileDialog,

    /// Open the file dialog for selecting the library directory.
    OpenLibraryFileDialog,
}

/// Screen for configuring application settings.
#[derive(Default)]
pub struct SettingsScreen {
    /// Dialog used to confirm deletion of a copy service.
    copy_service_dialog: Option<ConfirmDeleteDialog<usize>>,

    /// Form data for editing a copy service configuration.
    ///
    /// Will only be `Some` while a copy service is being edited. Only once service can be edited
    /// at a time.
    copy_service_form: Option<CopyServiceForm>,

    /// Indicates if the file dialog is open.
    file_dialog_open: bool,
}

impl SettingsScreen {
    /// Creates a new [`SettingsScreen`] instance.
    pub fn new() -> Self {
        Self {
            copy_service_dialog: None,
            copy_service_form: None,
            file_dialog_open: false,
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

    /// Generates a UI element for displaying a dialog.
    ///
    /// Will only return `Some` if the settings screen is displaying a dialog.
    pub fn dialog(&self) -> Option<Element<'_>> {
        if self.file_dialog_open {
            return Some(Container::new(Space::with_width(0)).into());
        }

        self.copy_service_dialog.as_ref()
            .map(|dialog| dialog.view(Message::DeleteCopyService { index: dialog.id }))
    }

    /// Callback when a dialog is closed.
    ///
    /// This will close any dialog this screen may have been opened without applying any changes.
    pub fn dialog_closed(&mut self) {
        self.copy_service_dialog = None;
        self.file_dialog_open = false;
    }

    /// Processes a settings screen message.
    pub fn process_message(
        &mut self,
        ctx: &Context,
        message: SettingsScreenMessage
    ) -> Task<Message> {
        match message {
            SettingsScreenMessage::AddCopyService => {
                if self.copy_service_form.is_none() {
                    self.copy_service_form = Some(CopyServiceForm::new(ctx.copy_services.len()));
                } else {
                    error!("ignore AddCopyService - already editing");
                }
            },
            SettingsScreenMessage::DeleteCopyService { index } => {
                let service = &ctx.copy_services[index];
                if self.copy_service_dialog.is_none() {
                    self.copy_service_dialog = Some(ConfirmDeleteDialog { 
                        id: index,
                        text: format!("{} ({})", service.name(), service.serial_number()),
                    });
                } else {
                    error!("ignore DeleteCopyService - already displayed");
                }
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
            SettingsScreenMessage::OpenArchiveFileDialog => {
                self.file_dialog_open = true;
                return Task::perform(
                    open_folder_dialog(), 
                    |path| {
                        match path {
                            Some(path) => Message::SetMediaArchivePath { path },
                            None => Message::CloseDialog,
                        }
                    }
                );
            },
            SettingsScreenMessage::OpenDataFileDialog => {
                self.file_dialog_open = true;
                return Task::perform(
                    open_folder_dialog(), 
                    |path| {
                        match path {
                            Some(path) => Message::SetDataPath { path },
                            None => Message::CloseDialog,
                        }
                    }
                );
            },
            SettingsScreenMessage::OpenInboxFileDialog => {
                self.file_dialog_open = true;
                return Task::perform(
                    open_folder_dialog(), 
                    |path| {
                        match path {
                            Some(path) => Message::SetMediaInboxPath { path },
                            None => Message::CloseDialog,
                        }
                    }
                );
            },
            SettingsScreenMessage::OpenLibraryFileDialog => {
                self.file_dialog_open = true;
                return Task::perform(
                    open_folder_dialog(), 
                    |path| {
                        match path {
                            Some(path) => Message::SetMediaLibraryPath { path },
                            None => Message::CloseDialog,
                        }
                    }
                );
            },
        }

        Task::none()
    }

    /// Generates the UI element for displaying the screen.
    pub fn view(&self, ctx: &Context) -> Element<'_> {
        let rows = Column::with_capacity(3)
            .push(self.appearance_view(ctx))
            // TODO: The following will need gaurds against making modifications while a copy or
            //       transcode operations being performed. Can get by modifying the config file 
            //       for development.
            // .push(self.path_view(ctx))
            // .push(self.copy_service_view(ctx))
            .max_width(1080)
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
                .width(Length::Fill)
                .padding([8, 0])
        }

        let header = Row::with_capacity(1)
            .push(text::heading2("Appearance"))
            .padding([8, 0]);

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

        let content = Column::with_capacity(6)
            .push(header)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .push(scale_factor)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .push(theme)
            .push(Rule::horizontal(1).class(RuleClass::Surface1));

        Container::new(content)
            .class(ContainerClass::Default)
            .into()
    }

    /// Generates the UI element for displaying copy service information.
    fn copy_service_item_view(&self, index: usize, copy_service: &CopyService) -> Element<'_> {
        let mut cols: Vec<Element<'_>> = Vec::with_capacity(3);

        cols.push(text::label(copy_service.name().to_owned())
            .width(Length::Fill)
            .into());
        cols.push(text::label(copy_service.serial_number().to_owned())
            .width(Length::Fill)
            .into());

        // Only show the edit controls if not currently editing a copy service configuration.
        //
        // TODO: Will also need to ensure that the copy service is not running a copy operation
        //       since editing a copy service mid operation could be bad. 
        if self.copy_service_form.is_none() {
            let edit_button = Button::new(ButtonClass::Default)
                .icon("fontawesome.v7.solid.edit")
                // .icon_size(20.0)
                .on_press(
                    Message::SettingsScreen(
                        SettingsScreenMessage::EditCopyService { index }
                    )
                );
            cols.push(edit_button.into());
        }

        Row::with_children(cols)
            .align_y(Alignment::Center)
            .padding([8, 0])
            .width(Length::Fill)
            .into()
    }

    /// Generates the UI element for displaying the section for the copy service settings.
    fn copy_service_view(&self, ctx: &Context) -> Element<'_> {
        let mut rows: Vec<Element<'_>> = Vec::with_capacity((ctx.copy_services.len() * 2) + 3);

        let header = Row::with_capacity(1)
            .push(text::heading2("Drives"))
            .padding([8, 0]);

        for (index, service) in ctx.copy_services.iter().enumerate() {
            rows.push(Rule::horizontal(1).class(RuleClass::Surface1).into());

            let row = match &self.copy_service_form {
                Some(form) => if form.index == index {
                    form.view()
                } else {
                    self.copy_service_item_view(index, service)
                },
                None => self.copy_service_item_view(index, service),
            };

            rows.push(row);
        }

        // Account for a new service being added.
        if let Some(form) = &self.copy_service_form && form.index >= ctx.copy_services.len() {
            if !ctx.copy_services.is_empty() {
                rows.push(Rule::horizontal(1).class(RuleClass::Surface1).into())
            }

            rows.push(form.view());
        }

        let content = Container::new(Column::with_children(rows))
            .class(ContainerClass::Default);

        let message = if self.copy_service_form.is_none() {
            Some(Message::SettingsScreen(SettingsScreenMessage::AddCopyService))
        } else {
            None
        };

        let controls = Row::with_capacity(1)
            .push(Button::new(ButtonClass::Primary)
                .icon("fontawesome.v7.solid.plus")
                .label("Add")
                .on_press_maybe(message))
            .padding([8, 0]);

        Column::with_capacity(4)
            .push(header)
            .push(content)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .push(controls)
            .spacing(0)
            .into()
    }

    /// Generates the UI element for displaying the section for the path (file system) settings.
    fn path_view(&self, ctx: &Context) -> Element<'_> {
        let header = Row::with_capacity(1)
            .push(text::heading2("File Paths"))
            .padding([8, 0]);

        fn form_row<'a>(
            label: &'a str,
            value: &Path,
            msg: SettingsScreenMessage
        ) -> Row<'a, crate::Message, Theme> 
        {
            let file_input = TextInput::new("", value.to_str().unwrap_or_default())
                .class(TextInputClass::Default)
                .line_height(iced::widget::text::LineHeight::Relative(1.45));

            let file_dialog_button = Button::new(ButtonClass::Default)
                .icon("fontawesome.v7.solid.ellipsis")
                .on_press(Message::SettingsScreen(msg));

            Row::with_capacity(2)
                .push(text::label(label).width(124))
                .push(file_input)
                .push(file_dialog_button)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .spacing(4)
                .padding([8, 0])
        }

        let inbox_row = form_row(
            "Media Inbox",
            &ctx.settings.fs.inbox,
            SettingsScreenMessage::OpenInboxFileDialog,
        );

        let library_row = form_row(
            "Media Library",
            &ctx.settings.fs.library,
            SettingsScreenMessage::OpenLibraryFileDialog,
        );

        let archive_row = form_row(
            "Media Archive",
            &ctx.settings.fs.archive,
            SettingsScreenMessage::OpenArchiveFileDialog,
        );

        let data_row = form_row(
            "Data",
            &ctx.settings.fs.data,
            SettingsScreenMessage::OpenDataFileDialog,
        );

        Column::with_capacity(10)
            .push(header)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .push(inbox_row)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .push(library_row)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .push(archive_row)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .push(data_row)
            .push(Rule::horizontal(1).class(RuleClass::Surface1))
            .spacing(0)
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

    /// Indicates if the service can be deleted.
    can_delete: bool,
}

impl CopyServiceForm {
    /// Creates a [`CopyServiceForm`] instance for adding a new service.
    pub fn new(index: usize) -> Self {
        Self {
            index,
            name: String::from(""),
            serial_number: String::from(""),
            can_apply: false,
            can_delete: false,
        }
    }

    /// Creates a [`CopyServiceForm`] instance for editing an existing service.
    pub fn from_service(index: usize, service: &CopyService) -> Self {
        Self {
            index,
            name: service.name().to_owned(),
            serial_number: service.serial_number().to_owned(),
            can_apply: false,
            can_delete: true,
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
        let input = TextInput::new("", &self.name)
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

        let input = TextInput::new("", &self.serial_number)
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

        let mut controls: Vec<Element<'_>> = Vec::with_capacity(4);
        
        if self.can_delete {
            let delete_button = Button::new(ButtonClass::Danger)
                .icon("fontawesome.v7.solid.trash")
                .label("Delete")
                .on_press(
                    Message::SettingsScreen(
                        SettingsScreenMessage::DeleteCopyService { index: self.index }
                    )
                );
            controls.push(delete_button.into());
        }

        controls.push(Space::with_width(Length::Fill).into());

        let apply_button = Button::new(ButtonClass::Success)
            .icon("fontawesome.v7.solid.check")
            .label("Save")
            .on_press_maybe(apply_message);
        controls.push(apply_button.into());

        let discard_button = Button::new(ButtonClass::Default)
            .icon("fontawesome.v7.solid.cancel")
            .label("Cancel")
            .on_press(Message::SettingsScreen( SettingsScreenMessage::EditCopyServiceDiscard));
        controls.push(discard_button.into());

        let controls = Row::with_children(controls)
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
                if self.name == service.name() && self.serial_number == service.serial_number() {
                    self.can_apply = false;
                    return;
                }
                continue;
            }

            if self.serial_number.eq_ignore_ascii_case(service.serial_number()) {
                self.can_apply = false;
                return;
            }
        }

        self.can_apply = true;
    }
}

/// Opens a dialog for selecting a folder.
async fn open_folder_dialog() -> Option<PathBuf> {
    let file = AsyncFileDialog::new()
        .pick_folder()
        .await;

    file.map(|handle| handle.path().to_owned())
}

// TESTING TODO:
// - Validation
// - Process Message
