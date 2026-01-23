// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the application window widget.

use glib::Object;

use gtk::{Application, HeaderBar, Stack, StackSwitcher};
use gtk::gio;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::prelude::*;

use crate::ui::context::ContextObject;
use crate::ui::widget::copy_page::CopyPageWidget;

glib::wrapper! {
    /// Application window widget.
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow,
                 gtk::Window,
                 gtk::Widget,
        @implements gio::ActionGroup,
                    gio::ActionMap,
                    gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Native,
                    gtk::Root,
                    gtk::ShortcutManager;
}

impl Window {
    /// Creates a new [`Window`] widget.
    pub fn new(app: &Application, context: &ContextObject) -> Self {
        Object::builder()
            .property("application", app)
            .property("context", context)
            .build()
    }

    /// Builds the user interface.
    ///
    /// It is expected that this will be called as part of the underlying widget's construction.
    /// See [`imp::Window::constructed`].
    fn build_ui(&self) {
        let context = self.context().expect("context not set");

        let copy_page = CopyPageWidget::new(&context);

        let transcode_page = gtk::Label::builder()
            .label("Transcode Page")
            .build();

        let catalog_page = gtk::Label::builder()
            .label("Catalog Page")
            .build();

        let stack = Stack::builder()
            .build();
        stack.add_titled(&copy_page, None, "Copy");
        stack.add_titled(&transcode_page, None, "Transcode");
        stack.add_titled(&catalog_page, None, "Catalog");

        let stack_switcher = StackSwitcher::builder()
            .stack(&stack)
            .build();

        let header_bar = HeaderBar::builder()
            .build();
        header_bar.pack_start(&stack_switcher);

        self.set_title(Some("Artie"));
        self.set_titlebar(Some(&header_bar));
        self.set_default_width(1080);
        self.set_default_height(920);
        self.set_child(Some(&stack));
    }
}

mod imp {
    //! Implemenation for the application window.

    use std::cell::RefCell;

    use gtk::ApplicationWindow;
    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::context::ContextObject;

    /// Implemenation for [`super::Window`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Window)]
    pub struct Window {
        /// The application context.
        #[property(get, set = Self::set_context, construct_only)]
        pub(super) context: RefCell<Option<ContextObject>>,
    }

    impl Window {
        /// Sets the application context.
        fn set_context(&self, context: Option<ContextObject>) {
            self.context.replace(context);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "ArtieApplicationWindow";
        type Type = super::Window;
        type ParentType = ApplicationWindow;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }
    }

    impl WidgetImpl for Window {}

    impl WindowImpl for Window {}

    impl ApplicationWindowImpl for Window {}
}
