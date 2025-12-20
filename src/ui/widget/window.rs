// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the application window widget.

use glib::Object;

use gtk::Application;
use gtk::gio;
use gtk::glib;

glib::wrapper! {
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
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }
}

mod imp {
    use gtk::{ ApplicationWindow, Button, CompositeTemplate };
    use gtk::glib;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::widget::copy_page::CopyPageWidget;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/example/artie/ui/window.ui")]
    pub struct Window {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "ArtieApplicationWindow";
        type Type = super::Window;
        type ParentType = ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            CopyPageWidget::ensure_type();

            klass.bind_template();
            // klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for Window {}

    impl WindowImpl for Window {}

    impl ApplicationWindowImpl for Window {}
}
