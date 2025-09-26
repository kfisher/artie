// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;

use iced::border::Border;
use iced::widget::Row;
use iced::widget::button::{Catalog, Status, Style};
use iced::widget::tooltip::{Position, Tooltip};

use crate::widget::container::{Container, ContainerClass};
use crate::Message;
use crate::theme::Theme;
use crate::theme::color::Color;
use crate::theme::palette::Palette;
use crate::widget::Element;
use crate::widget::icon;
use crate::widget::text::Text;

/// The style classes used for the button widgets.
#[derive(Default)]
pub enum ButtonClass {
    /// The default style of buttons.
    #[default]
    Default,

    /// Style for the buttons used for navigation.
    Nav(bool),

    /// Button whose background is the theme's primary color.
    Primary,
}

impl ButtonClass {
    /// Styles a button in the disabled state based on the class using the provided color palette.
    pub fn disabled(&self, palette: &Palette) -> Style {
        Style {
            text_color: palette.primary.alpha(0.5).into(),
            ..self.normal(palette)
        }
    }

    /// Styles a button in the hovered state based on the class using the provided color palette.
    pub fn hovered(&self, palette: &Palette) -> Style {
        let filter = match palette.is_dark {
            true => Color::lighten,
            false => Color::darken,
        };
        match self {
            ButtonClass::Default => Style {
                background: Some(palette.overlay_0.alpha(0.25).into()),
                ..self.normal(palette)
            },
            ButtonClass::Nav(selected) => {
                if *selected {
                    self.normal(palette)
                } else {
                    Style {
                        background: Some(palette.primary.alpha(0.25).into()),
                        border: Border::default().rounded(4),
                        text_color: palette.primary.into(),
                        ..self.normal(palette)
                    }
                }
            },
            ButtonClass::Primary => Style {
                background: Some(filter(palette.primary, 0.05).into()),
                ..self.normal(palette)
            },
        }
    }

    /// Styles a button in the active state based on the class using the provided color palette.
    pub fn normal(&self, palette: &Palette) -> Style {
        match self {
            ButtonClass::Default => Style {
                background: Some(palette.surface_0.into()),
                border: Border::default(),
                text_color: palette.text.into(),
                ..Style::default()
            },
            ButtonClass::Nav(selected) => {
                if *selected {
                    Style { 
                        background: Some(palette.primary.alpha(0.25).into()),
                        border: Border::default().rounded(4),
                        text_color: palette.primary.into(),
                        ..Style::default() 
                    }
                } else {
                    Style { 
                        background: None,
                        border: Border::default(),
                        text_color: palette.text.into(),
                        ..Style::default() 
                    }
                }
            },
            ButtonClass::Primary => Style {
                background: Some(palette.primary.into()),
                border: Border::default(),
                text_color: palette.text.into(),
                ..Style::default()
            },
        }
    }

    /// Styles a button in the pressed state based on the class using the provided color palette.
    pub fn pressed(&self, palette: &Palette) -> Style {
        let filter = match palette.is_dark {
            true => Color::lighten,
            false => Color::darken,
        };
        match self {
            ButtonClass::Default => Style {
                ..self.normal(palette)
            },
            ButtonClass::Nav(_) => Style {
                text_color: filter(palette.primary, 0.05).into(),
                ..self.normal(palette)
            },
            ButtonClass::Primary => Style {
                background: Some(filter(palette.primary, 0.10).into()),
                ..self.normal(palette)
            },
        }
    }
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
            let text = Text::new(label);
            content.push(text.into());
        }

        let content = Row::with_children(content)
            .padding(button.padding);

        let widget = iced::widget::button::Button::new(content)
            .padding(0)
            .class(button.class)
            .on_press_maybe(button.on_press);

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
        match status {
            Status::Active => class.normal(palette),
            Status::Hovered => class.hovered(palette),
            Status::Pressed => class.pressed(palette),
            Status::Disabled => class.disabled(palette),
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

