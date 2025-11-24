// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;
use std::rc::Rc;
use std::time::Duration;

use iced::{Alignment, Border, Length};
use iced::font::{Family, Font, Weight};
use iced::widget::{Column, Row, Space};
use iced::widget::container::Style as ContainerStyle;

use crate::drive::{DiscState, Drive, DriveState};
use crate::ui::{Element, Message};
use crate::ui::theme::Theme;
use crate::ui::widgets::animation::RotationAnimation;
use crate::ui::widgets::button::{Button, ButtonClass};
use crate::ui::widgets::container::{Container, ContainerClass};
use crate::ui::widgets::progress_bar::ProgressBar;
use crate::ui::widgets::rule::{Rule, RuleClass};
use crate::ui::widgets::text::Text;
use crate::ui::widgets::icon::{self};

use super::CopyScreenMessage;
use super::form::CopyForm;

/// UI component used to control copying media for an optical drive.
pub struct DriveComponent {
    /// The optical drive associated with this component.
    drive: Rc<Drive>,

    /// Form used to input the copy parameters.
    pub form: CopyForm,

    /// The index of the component within the list of components.
    ///
    /// Used to identify the specific component instance, mainly for messaging purposes.
    /// 
    /// TODO: Should either replace with using the serial number or include the serial number in 
    ///       the messages as a sanity check to protect against something changing between the
    ///       message being sent and processed.
    pub index: usize,
    
    /// Animation used for rotating the disc icon when copying.
    disc_animation: RotationAnimation,
}

impl DriveComponent {
    /// Creates a new [`DriveComponent`] instance.
    pub fn new(drive: &Rc<Drive>, index: usize) -> Self {
        Self {
            drive: drive.clone(),
            form: CopyForm::new(),
            index,
            disc_animation: RotationAnimation::new(1.0),
        }
    }

    /// Tick the component (used to update animations).
    pub fn tick(&mut self, delta_time: f32) {
        if self.disc_animation.enabled {
            self.disc_animation.tick(delta_time);
        }
    }

    /// Returns true if this component requires tick to be active.
    pub fn should_tick(&self) -> bool {
        self.disc_animation.enabled
    }

    /// Generates the UI element for the drive component.
    pub fn view(&self) -> Element<'_> {

        let mut content: Vec<Element<'_>> = Vec::with_capacity(5);

        match self.drive.state {
            DriveState::Disconnected => {
                content.push(self.default_header());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.disconnected_content());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.default_footer());
            },
            DriveState::Idle { .. } => {
                content.push(self.idle_header());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.idle_content());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.default_footer());
            },
            DriveState::Copying { .. } => {
                content.push(self.copying_header());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.copying_content());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.default_footer());
            },
            DriveState::Success => {
                content.push(self.reset_header());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.success_content());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.default_footer());
            },
            DriveState::Failed { .. } => {
                content.push(self.reset_header());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.failed_content());
                content.push(Rule::horizontal(1).class(RuleClass::Surface1).into());
                content.push(self.default_footer());
            },
        }

        let content = Column::with_children(content)
            .padding(1);

        Container::new(content)
            .class(ContainerClass::Custom(|theme| ContainerStyle {
                background: Some(theme.palette().surface_1.color.into()),
                border: Border::default()
                    .width(1)
                    .color(theme.palette().surface_1.border)
                    .rounded(2),
                ..ContainerStyle::default()
            }))
            .max_width(1080)
            .into()
    }

    //---------------------------------------------------------------------
    //---------------------------------------------------------------------

    /// Generates the header UI element for the drive component when the drive is in the
    /// [`DriveState::Copying`] state.
    fn copying_header(&self) -> Element<'_> {
        // let message = Message::CancelCopyDisc { index: component.index };

        let button = Button::new(ButtonClass::Danger)
            .icon("fontawesome.v7.solid.ban")
            .label("Cancel");
            // .on_press(message);

        let controls = vec![ 
            button.into(), 
        ];

        self.header_component(controls)
    }

    /// Generates the header UI element for the drive component when the drive is in the
    /// [`DriveState::Copying`] state.
    /// 
    /// # Panics
    /// 
    /// This will panic if the drive is not in the copying state.
    fn copying_content(&self) -> Element<'_> {
        let drive = &self.drive;
        let DriveState::Copying { 
            stage,
            task,
            subtask,
            task_progress,
            subtask_progress,
            elapsed_time,
        } = &drive.state else {
            // should be safe to expect state to be copying
            panic!("Expected copying state")
        };

        let icon = rotated_icon(
            "fontawesome.v7.solid.compact-disc", 
            self.disc_animation.degrees, 
            ContainerClass::Secondary
        );

        let header = Row::with_capacity(4)
            .push(Text::new(stage))
            .push(Space::with_width(Length::Fill))
            .push(Text::new(format_duration(elapsed_time)));

        let task_title = progress_text(task);
        let task_progress_bar = ProgressBar::new(0.0..=1.0, *task_progress).girth(16);
        let task = Column::with_capacity(2)
            .push(task_title)
            .push(task_progress_bar)
            .spacing(2);

        let subtask_title = progress_text(subtask);
        let subtask_progress_bar = ProgressBar::new(0.0..=1.0, *subtask_progress).girth(16);
        let subtask = Column::with_capacity(2)
            .push(subtask_title)
            .push(subtask_progress_bar)
            .spacing(2);

        let content = Column::with_capacity(5)
            .push(header)
            .push(task)
            .push(subtask)
            .spacing(8);

        let content = Container::new(content)
            .padding(8)
            .into();

        self.content_component(icon, content)
    }

    /// Generates the default header UI element for the drive component.
    fn default_header(&self) -> Element<'_> {
        self.header_component(Vec::default())
    }

    /// Generates the footer UI element for the drive component.
    fn default_footer(&self) -> Element<'_> {
        self.footer_component()
    }

    /// Generates the header UI element for the drive component when the drive is in the
    /// [`DriveState::Success`] state.
    ///
    /// # Panics
    /// 
    /// This will panic if the drive is not in the failed state.
    fn failed_content(&self) -> Element<'_> {
        let drive = &self.drive;
        let DriveState::Failed { error } = drive.state.clone() else {
            // This function should only ever be called if the drive's state is failed.
            panic!("drive not in failed state")
        };

        let icon = icon(Some("fontawesome.v7.solid.exclamation-triangle"), ContainerClass::Danger);

        let error_message = Container::new(Text::new(error))
            .class(ContainerClass::Custom(|theme| ContainerStyle {
                background: Some(theme.palette().surface_3.color.into()),
                border: Border::default()
                    .width(1)
                    .color(theme.palette().surface_3.border)
                    .rounded(2),
                ..ContainerStyle::default()
            }))
            .width(Length::Fill)
            .padding(8);

        let content = Column::with_capacity(2)
            .push(Text::new("The copy operation failed to complete."))
            .push(error_message)
            .spacing(8);

        let content = Container::new(content)
            .padding(16)
            .width(Length::Fill)
            .into();

        self.content_component(icon, content)
    }

    /// Generates the footer UI element for the drive component.
    fn footer_component(&self) -> Element<'_> {
        let drive = &self.drive;
        if drive.state == DriveState::Disconnected {
            return Row::with_capacity(1)
                .push(footer_text(" "))
                .padding([4, 12])
                .width(Length::Fill)
                .into()
        };

        let disc_text = match &drive.disc {
            DiscState::None => footer_text("No Disc"),
            DiscState::Inserted { label, uuid: _ } => footer_text(label),
        };

        let drive_text = footer_text(format!("[ {} ][ {} ]", drive.path, drive.serial_number));

        Row::with_capacity(3)
            .push(disc_text)
            .push(Space::with_width(Length::Fill))
            .push(drive_text)
            .padding([4, 12])
            .width(Length::Fill)
            .into()
    }

    /// Generates the header UI element for the drive component.
    fn header_component<'a>(&self, controls: Vec<Element<'a>>) -> Element<'a> {
        let mut header: Vec<Element<'_>> = Vec::with_capacity(2 + controls.len());

        let name = Text::new(self.drive.name.clone())
            .size(20)
            .font(Font {
                weight: Weight::Bold,
                ..Font::default()
            });
        header.push(name.into());
        header.push(Space::with_width(Length::Fill).into());
        header.extend(controls);

        Row::with_children(header)
            .padding([4,8])
            .width(Length::Fill)
            .height(44)
            .align_y(Alignment::Center)
            .spacing(8)
            .into()
    }

    /// Generates the UI element for the content of the drive component when the drive is in the 
    /// [`DriveState::Idle`] state.
    fn idle_content(&self) -> Element<'_> {
        let drive = &self.drive;
        match drive.disc {
            DiscState::None => {
                let icon = icon(None, ContainerClass::Secondary);
                let content = message_content("The drive is empty");
                self.content_component(icon, content)
            },
            DiscState::Inserted { .. } => {
                let icon = icon(Some("fontawesome.v7.solid.compact-disc"), ContainerClass::Secondary);
                let content = self.form.view(self.index);
                self.content_component(icon, content)
            }
        }
    }

    /// Generates the header UI element for the drive component when the drive is in the
    /// [`DriveState::Idle`] state.
    fn idle_header(&self) -> Element<'_> {
        let drive = &self.drive;
        if drive.disc == DiscState::None {
            return self.default_header();
        }

        let message = if self.form.valid() {
            None
            // FIXME
            // Some(Message::CopyDisc { index: component.index })
        } else {
            None
        };

        let copy_button = Button::new(ButtonClass::Primary)
            .icon("fontawesome.v7.solid.file-import")
            .label("Copy")
            .on_press_maybe(message);

        let message = CopyScreenMessage::ClearCopyForm { index: self.index };

        let clear_button = Button::new(ButtonClass::Default)
            .icon("fontawesome.v7.solid.xmark-circle")
            .label("Clear")
            .on_press(message.into());

        let controls = vec![ 
            clear_button.into(),
            copy_button.into(), 
        ];

        self.header_component(controls)
    }

    //---------------------------------------------------------------------
    //---------------------------------------------------------------------

    /// Generates the UI element for the drive component's content section.
    fn content_component<'a>(&self, icon: Element<'a>, content: Element<'a>) -> Element<'a> {
        let content = Row::with_capacity(3)
            .push(icon)
            .push(Rule::vertical(1).class(RuleClass::Surface1))
            .push(content)
            .align_y(Alignment::Center)
            .height(128);

        Container::new(content)
            .width(Length::Fill)
            .into()
    }

    /// Generates the UI element for the content of the drive component when the drive is in the 
    /// [`DriveState::Disconnected`] state.
    fn disconnected_content(&self) -> Element<'_> {
        let icon = icon(Some("fontawesome.v7.solid.plug-circle-minus"), ContainerClass::Surface1);
        let content = message_content("The drive is currently unavailable.");
        self.content_component(icon, content)
    }

    /// Generates the header UI element for the drive component when the drive is in the
    /// [`DriveState::Success`] or [`State::Failed`] states.
    fn reset_header(&self) -> Element<'_> {
        // let message = Message::ResetCopyService { index: component.index };

        let reset_button = Button::new(ButtonClass::Primary)
            .icon("fontawesome.v7.solid.rotate-left")
            .label("Reset");
            // .on_press(message);

        let controls = vec![ 
            reset_button.into(),
        ];

        self.header_component(controls)
    }

    /// Generates the header UI element for the drive component when the drive is in the
    /// [`DriveState::Success`] state.
    fn success_content(&self) -> Element<'_> {
        let icon = icon(Some("fontawesome.v7.solid.circle-check"), ContainerClass::Success);
        let content = message_content("The copy operation has completed successfully.");
        self.content_component(icon, content)
    }
}

/// Generates the UI element for text added to the drive component's footer.
fn footer_text<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .size(12)
        .font(Font {
            weight: Weight::Normal,
            family: Family::Monospace,
            ..Font::default()
        })
}

/// Formats duration.
fn format_duration(duration: &Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Generates the UI element for the component's icon component.
fn icon(icon: Option<&str>, class: ContainerClass) -> Element<'_>
{
    let icon: Element<'_> = match icon {
        Some(name) => icon::text(name)
            .width(90)
            .height(90)
            .into(),
        None => Space::new(Length::Fill, Length::Fill).into(),
    };

    Container::new(icon)
        .class(class)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(128)
        .height(128)
        .into()
}

/// Generates the UI element for the component's icon component with a rotated icon.
fn rotated_icon(icon: &str, degrees: f32, class: ContainerClass) -> Element<'_> {
    let icon = icon::text(icon)
        .width(94)
        .height(94)
        .rotation(degrees);

    Container::new(icon)
        .class(class)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(128)
        .height(128)
        .into()
}

/// Generates the UI element for displaying text in the content section of the drive component.
fn message_content<'a, T>(text: T) -> Element<'a> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Container::new(Text::new(text.into()))
        .padding(16)
        .width(Length::Fill)
        .into()
}

/// Generates the UI element for text added to the drive component's footer.
fn progress_text<'a, T>(text: T) -> Text<'a, Theme> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Text::new(text.into())
        .size(12)
        .font(Font {
            weight: Weight::Normal,
            family: Family::Monospace,
            ..Font::default()
        })
}


