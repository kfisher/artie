// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application UI.

mod context;
mod data;
mod helpers;
mod widget;

use gtk::gdk::Display;
use gtk::gio::prelude::*;
use gtk::gio;
use gtk::glib;
use gtk::prelude::GtkWindowExt;
use gtk::{Application, CssProvider, IconTheme};

use crate::{Mode, Result};

use crate::bus::Handle;
use context::ContextObject;
use widget::Window;

/// Messages used to send requests to the UI.
#[derive(Debug)]
pub enum Message {
}

/// Runs the UI.
///
/// This will block until the application is closed.
///
/// # Args
///
/// `mode`:  The mode the application is being run in. When running in worker mode, the UI will be
/// much more limited since it is the control that the user is expected to interact with.
pub fn run(mode: Mode, bus: &Handle) -> Result<glib::ExitCode> {
    gio::resources_register_include!("compiled.gresource")?;

    let context = ContextObject::new(mode, bus.clone());

    let app = Application::builder()
        // TODO: Comment this out for now for testing so that we can create multiple instances of
        //       the application to test networking.
        // TODO: Since single instance is the default, may need to revist some things to avoid
        //       extra instances of managers (e.g. client manager) if a second instance is started
        //       which seems to run some code on the single instance.
        //.application_id(APP_ID)
        .build();
    app.connect_activate(move |app| {
        build(app, context.clone());
    });

    // Override the command line arguments. Otherwise, GTK will generate errors for the command
    // line arguments defined above. May need to revist to support GTK arguments.
    Ok(app.run_with_args(&["artie"]))
}

/// Builds the application window.
/// 
/// This is used as the callback GTK when the application's widgets whould be constructed.
///
/// # Args
///
/// `app`:  The GTK application being built.
///
/// `context`: The UI's application context.
fn build(app: &Application, context: ContextObject) {
    let css_provider = CssProvider::new();
    css_provider.load_from_resource("org/example/artie/css/app.css");

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let icon_theme = IconTheme::for_display(&Display::default().unwrap());
    icon_theme.add_resource_path("/org/example/artie/icons");

    let window = Window::new(app, &context);
    window.present();
}

