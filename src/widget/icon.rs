// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

// Portions of the code within this file were adapted from the iced project's SVG implementation.
// File: https://github.com/iced-rs/iced/blob/master/widget/src/svg.rs, Commit: 31bc6d4
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MIT

//! SVG Icons

use iced::{ContentFit, Degrees, Length, Point, Rectangle, Rotation, Size, Vector};
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::mouse::Cursor;
use iced::advanced::renderer::Style;
use iced::advanced::svg::{Renderer as SvgRenderer, Svg};
use iced::advanced::widget::{Tree, Widget};
use iced::widget::svg::Handle;

use crate::Message;
use crate::theme::Theme;
use crate::widget::Element;

/// The style classes used for the icon widget.
#[derive(Default)]
pub enum IconClass {
    /// Default style that does not apply any filters to the icon's appearance.
    #[default]
    Default,

    /// Icon's color will match the text color of its parent.
    Text,
}

/// An SVG based icon.
pub struct Icon {
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    rotation: Rotation,
    class: IconClass,
    opacity: f32,
}

impl Icon {
    /// Creates a new [`Icon`] from the given [`Handle`].
    pub fn new<T>(handle: T) -> Self 
    where 
        T: Into<Handle>
    {
        Self {
            handle: handle.into(),
            width: Length::Fill,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
            class: IconClass::default(),
            rotation: Rotation::default(),
            opacity: 1.0,
        }
    }

    /// Consumes the icon returning a modified version with the provided class.
    pub fn class(mut self, class: IconClass) -> Self {
        self.class = class;
        self
    }

    /// Consumes the icon returning a modified version with the provided height.
    pub fn height<T>(mut self, height: T) -> Self 
    where 
        T: Into<Length> + Copy
    {
        self.height = height.into();
        self
    }

    /// Consumes the icon returning a modified version with the provided width.
    pub fn width<T>(mut self, width: T) -> Self 
    where 
        T: Into<Length> + Copy
    {
        self.width = width.into();
        self
    }

    /// Consumes the icon returning a modified version with the provided rotation.
    pub fn rotation(mut self, degrees: f32) -> Self 
    {
        self.rotation = Rotation::Floating(Degrees(degrees).into());
        self
    }
}

impl<Renderer> Widget<Message, Theme, Renderer> for Icon 
where 
    Renderer: SvgRenderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(&mut self, _tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        // The raw w/h of the underlying image
        let Size { width, height } = renderer.measure_svg(&self.handle);
        let image_size = Size::new(width as f32, height as f32);

        // The rotated size of the svg
        let rotated_size = self.rotation.apply(image_size);

        // The size to be available to the widget prior to `Shrink`ing
        let raw_size = limits.resolve(self.width, self.height, rotated_size);

        // The uncropped size of the image when fit to the bounds above
        let full_size = self.content_fit.fit(rotated_size, raw_size);

        // Shrink the widget to fit the resized image, if requested
        let final_size = Size {
            width: match self.width {
                Length::Shrink => f32::min(raw_size.width, full_size.width),
                _ => raw_size.width,
            },
            height: match self.height {
                Length::Shrink => f32::min(raw_size.height, full_size.height),
                _ => raw_size.height,
            },
        };

        Node::new(final_size)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let Size { width, height } = renderer.measure_svg(&self.handle);
        let image_size = Size::new(width as f32, height as f32);
        let rotated_size = self.rotation.apply(image_size);

        let bounds = layout.bounds();
        let adjusted_fit = self.content_fit.fit(rotated_size, bounds.size());
        let scale = Vector::new(
            adjusted_fit.width / rotated_size.width,
            adjusted_fit.height / rotated_size.height,
        );

        let final_size = image_size * scale;

        let position = match self.content_fit {
            ContentFit::None => Point::new(
                bounds.x + (rotated_size.width - adjusted_fit.width) / 2.0,
                bounds.y + (rotated_size.height - adjusted_fit.height) / 2.0,
            ),
            _ => Point::new(
                bounds.center_x() - final_size.width / 2.0,
                bounds.center_y() - final_size.height / 2.0,
            ),
        };

        let drawing_bounds = Rectangle::new(position, final_size);

        let color = match self.class {
            IconClass::Default => None,
            IconClass::Text => Some(style.text_color),
        };

        let render = |renderer: &mut Renderer| {
            renderer.draw_svg(
                Svg {
                    handle: self.handle.clone(),
                    color,
                    rotation: self.rotation.radians(),
                    opacity: self.opacity,
                },
                drawing_bounds,
            );
        };

        if adjusted_fit.width > bounds.width
            || adjusted_fit.height > bounds.height
        {
            renderer.with_layer(bounds, render);
        } else {
            render(renderer);
        }
    }
}

impl<'a> From<Icon> for Element<'a> {
    fn from(icon: Icon) -> Self {
        Element::new(icon)
    }
}

/// Creates an icon without any modifications to appearance.
///
/// The `name` is the name of the icon which is the name of the file in the "resources/icons"
/// folder without the file extension.
pub fn default(name: &str) -> Icon {
    let path = format!("{}/resources/icons/{}.svg", env!("CARGO_MANIFEST_DIR"), name);
    Icon::new(path)
}

/// Creates an icon whose color will match the text color of the icon's parent element.
///
/// The `name` is the name of the icon which is the name of the file in the "resources/icons"
/// folder without the file extension.
pub fn text(name: &str) -> Icon {
    default(name).class(IconClass::Text)
}
