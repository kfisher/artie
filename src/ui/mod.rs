// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the graphical user interface.

pub mod context;
pub mod widget;

use gtk::prelude::GtkWindowExt;
use gtk::{Application, CssProvider, IconTheme};
use gtk::gdk::Display;

use widget::Window;

pub use context::ContextObject;

/// Builds the application window.
pub fn build(app: &Application) {
    let css_provider = CssProvider::new();
    css_provider.load_from_resource("org/example/artie/css/app.css");

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let icon_theme = IconTheme::for_display(&Display::default().unwrap());
    icon_theme.add_resource_path("/org/example/artie/icons");

    let context = ContextObject::builder()
        .build()
        .expect("Failed to create application context");

    let window = Window::new(app, &context);
    window.present();

    //--] let menu_popover = PopoverMenu::builder()
    //--]     .build();

    //--] let menu_button = MenuButton::builder()
    //--]     .icon_name("open-menu-symbolic")
    //--]     .popover(&menu_popover)
    //--]     .build();

    //--] header_bar.pack_end(&menu_button);
}

pub mod helpers {
    use gtk::Entry;
    use gtk::prelude::*;

    pub const INVALID_CSS_CLASS: &str = "invalid";

    // TODO
    pub fn update_validity_style(entry: &Entry, valid: bool) {
        if valid {
            entry_valid(entry);
        } else {
            entry_invalid(entry);
        }
    }

    /// Marks the entry as valid.
    ///
    /// This will remove the invalid css class (see: [`INVALID_CSS_CLASS`])
    pub fn entry_valid(entry: &Entry) {
        entry.remove_css_class(INVALID_CSS_CLASS);
    }

    /// Marks the entry as invalid.
    ///
    /// This will add the invalid css class (see: [`INVALID_CSS_CLASS`])
    pub fn entry_invalid(entry: &Entry) {
        entry.add_css_class(INVALID_CSS_CLASS);
    }

}
