// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the graphical user interface.

mod page;

use gtk::{
    Application,
    ApplicationWindow,
    Box,
    Button,
    HeaderBar,
    Orientation,
    MenuButton,
    PopoverMenu,
    Stack,
    StackSwitcher,
};

use gtk::prelude::{BoxExt, GtkWindowExt, WidgetExt};


/// Builds the application window.
pub fn build(app: &Application) {
    let stack = Stack::builder()
        .build();

    let copy_page = page::copy::build();
    stack.add_titled(&copy_page, None, "Copy");

    let transcode_page = page::transcode::build();
    stack.add_titled(&transcode_page, None, "Transcode");

    let catalog_page = page::catalog::build();
    stack.add_titled(&catalog_page, None, "Catalog");

    let stack_switcher = StackSwitcher::builder()
        .stack(&stack)
        .build();

    let menu_popover = PopoverMenu::builder()
        .build();

    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .popover(&menu_popover)
        .build();

    let header_bar = HeaderBar::builder()
        .build();
    header_bar.pack_start(&stack_switcher);
    header_bar.pack_end(&menu_button);

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&stack)
        .default_width(1080)
        .default_height(920)
        .title("Artie")
        .titlebar(&header_bar)
        .build();

    window.present();
}

