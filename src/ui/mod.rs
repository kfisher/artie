// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use gtk::{Application, ApplicationWindow, Button};
use gtk::prelude::*;

pub fn build(app: &Application) {
   // Create a button with label and margins
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(|button| {
        // Set the label to "Hello World!" after the button has been clicked on
        button.set_label("Hello World!");
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Artie")
        .child(&button)
        .default_width(1080)
        .default_height(920)
        .build();

    let settings = window.settings();
    settings.set_gtk_xft_dpi(settings.gtk_xft_dpi() * 2);

    // Present window
    window.present();
}
