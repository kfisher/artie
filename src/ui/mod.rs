// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the graphical user interface.

pub mod context;
pub mod widget;

use gtk::prelude::GtkWindowExt;
use gtk::{Application, CssProvider};
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

