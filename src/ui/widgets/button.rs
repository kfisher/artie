// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Buttons

use std::borrow::Cow;

use iced::{Alignment, Length};
use iced::border::Border;
use iced::widget::Row;
use iced::widget::button::{Catalog, Status, Style};
use iced::widget::tooltip::{Position, Tooltip};

use crate::ui::{Element, Message};
use crate::ui::theme::Theme;
use crate::ui::theme::palette::{ColorSet, Palette};

use super::container::{Container, ContainerClass};
use super::icon;
use super::text::{Text, TextClass};

/// The style classes used for the button widgets.
#[derive(Default)]
pub enum ButtonClass {
    /// Style for buttons used for destructive actions.
    Danger,

    /// The default style of buttons.
    #[default]
    Default,

    /// Style for the buttons used for navigation.
    Nav(bool),

    /// Style for buttons using the primary application color.
    Primary,

    /// Style for buttons used for afrimative actions.
    Success,
}

/// Widget that emits a message when clicked.
pub struct Button<'a> {
    /// The button's style class which controls its appearance.
    class: ButtonClass,

    /// Name of the icon.
    icon: Option<Cow<'a, str>>,

    /// Size of the icon in pixels.
    icon_size: f32,

    /// Button text.
    label: Option<Cow<'a, str>>,

    /// Button padding on all four sides for the button.
    padding: f32,

    /// Message emitted when the button is pressed.
    on_press: Option<Message>,

    /// Optional tooltip to display when mousing over.
    tooltip: Option<Cow<'a, str>>,

    /// The width of the button.
    width: Length,
}

impl<'a> Button<'a> {
    /// Creates a [`Button`] instance with the provided style class.
    pub fn new(class: ButtonClass) -> Self {
        Button {
            class,
            icon: None,
            icon_size: 16.0,
            label: None,
            padding: 8.0,
            on_press: None,
            tooltip: None,
            width: Length::Shrink,
        }
    }

    /// Sets the icon given the icon's name.
    ///
    /// The `name` is the name of the icon which is the name of the file in the "resources/icons"
    /// folder without the file extension.
    pub fn icon<T>(mut self, name: T) -> Self 
    where 
        T: Into<Cow<'a, str>>
    {
        self.icon = Some(name.into());
        self
    }

    /// Sets the icon's size.
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self.padding = size / 2.0;
        self
    }

    /// Sets the button's text.
    pub fn label<T>(mut self, label: T) -> Self 
    where 
        T: Into<Cow<'a, str>>
    {
        self.label = Some(label.into());
        self
    }

    /// Set the message to emit when clicked.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    /// Set the message to emit when clicked. If `None`, the button will not emit a message and is
    /// considered disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press;
        self
    }

    /// Sets the tooltip to display when mousing over the button.
    pub fn tooltip(mut self, tooltip: Cow<'a, str>) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    /// Sets the width of the button.
    pub fn width<T>(mut self, width: T) -> Self 
    where 
        T: Into<Length>
    {
        self.width = width.into();
        self
    }
}

impl<'a> From<Button<'a>> for Element<'a> {
    fn from(button: Button<'a>) -> Self {
        let mut content: Vec<Element<'_>> = Vec::with_capacity(2);

        if let Some(name) = button.icon {
            let icon = icon::text(&name)
                .width(button.icon_size)
                .height(button.icon_size);
            content.push(icon.into());
        }

        if let Some(label) = button.label {
            let text = Text::new(label).class(TextClass::Inherit)
                .width(Length::Fill)
                .center();
            content.push(text.into());
        }

        let content = Row::with_children(content)
            .padding(button.padding)
            .spacing(button.padding)
            .align_y(Alignment::Center);

        let widget = iced::widget::button::Button::new(content)
            .padding(0)
            .class(button.class)
            .on_press_maybe(button.on_press)
            .width(button.width);

        if let Some(tooltip) = button.tooltip {
            Tooltip::new(
                widget,
                Container::new(iced::widget::text(tooltip).size(10)),
                Position::Right,
            ).class(ContainerClass::Tooltip).gap(4).into()
        } else {
            widget.into()
        }
    }
}

impl Catalog for Theme {
    type Class<'a> = ButtonClass;

    fn default<'a>() -> Self::Class<'a> {
        ButtonClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        let palette = self.palette();

        match class {
            ButtonClass::Danger => button_style(
                &palette.danger,
                &status,
            ),
            ButtonClass::Default => button_style(
                &palette.surface_3,
                &status,
            ),
            ButtonClass::Nav(selected) => nav_button_style(*selected, palette, &status),
            ButtonClass::Primary => button_style(
                &palette.primary,
                &status,
            ),
            ButtonClass::Success => button_style(
                &palette.success,
                &status,
            ),
        }
    }
}

/// Creates a navigation button.
pub fn nav_button<'a, T>(icon: T, message: Message, tooltip: T, active: bool) -> Element<'a> 
where 
    T: Into<Cow<'a, str>> + 'a
{
    Button::new(ButtonClass::Nav(active))
        .icon(icon)
        .on_press(message)
        .tooltip(tooltip.into())
        .into()
}

/// Generates the style for the default type of buttons.
fn button_style(color_set: &ColorSet, status: &Status) -> Style {
    let background = match status {
        Status::Active | Status::Pressed => color_set.color,
        Status::Hovered => color_set.hover,
        Status::Disabled => color_set.color.scale_alpha(0.50),
    };

    Style {
        background: Some(background.into()),
        border: Border::default().width(0).rounded(2),
        text_color: color_set.text,
        ..Style::default()
    }
}

/// Generates the style for navigation type buttons.
fn nav_button_style(selected: bool, palette: &Palette, status: &Status) -> Style {
    let base = match selected {
        true => Style {
            background: Some(palette.primary.color.into()),
            border: Border::default().rounded(4),
            text_color: palette.primary.text,
            ..Style::default() 
        },
        false => Style {
            background: None,
            border: Border::default(),
            text_color: palette.text.color,
            ..Style::default() 
        },
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => if selected {
            base
        } else {
            Style {
                background: Some(palette.primary.color.into()),
                border: Border::default().rounded(4),
                text_color: palette.primary.text,
                ..base
            }
        },
        Status::Disabled => Style {
            background: Some(palette.primary.color.scale_alpha(0.50).into()),
            ..base
        },
    }
}

