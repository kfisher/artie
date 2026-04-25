// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Optical drive widget.
//!
//! The optical drive widget is used to initiate, monitor, and terminate copy operations.

use gtk::{
    Align,
    Box,
    Image,
    Label,
    ProgressBar,
    Orientation,
    Stack
};
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::drive::FormDataUpdate;
use crate::ui::data::{OpticalDriveState, OpticalDriveObject};
use crate::ui::widget::{CopyFormWidget, IconButton};
use crate::task;

glib::wrapper! {
    pub struct DriveWidget(ObjectSubclass<imp::DriveWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl DriveWidget {
    /// Creates a new drive widget instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::DriveWidget`]) when constructed.
    fn build_ui(&self) {
        let imp = self.imp();

        let name_label = imp.name_label
            .borrow()
            .clone();
        name_label.add_css_class("drive-widget-label");
        name_label.set_halign(Align::Start);
        name_label.set_hexpand(true);

        let clear_button = IconButton::new(
            "fontawesome.v7.solid.xmark-circle-symbolic",
            "Clear",
        );
        clear_button.add_css_class("default");

        let copy_button = IconButton::new(
            "fontawesome.v7.solid.file-import-symbolic",
            "Copy",
        );
        copy_button.add_css_class("secondary");

        let reset_button = IconButton::new(
            "fontawesome.v7.solid.rotate-left-symbolic",
            "Reset",
        );
        reset_button.add_css_class("primary");

        let cancel_button = IconButton::new(
            "fontawesome.v7.solid.ban-symbolic",
            "Cancel",
        );
        cancel_button.add_css_class("danger");

        let header_row = Box::builder()
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();
        header_row.append(&name_label);
        header_row.append(&clear_button);
        header_row.append(&copy_button);
        header_row.append(&reset_button);
        header_row.append(&cancel_button);
        header_row.add_css_class("drive-widget-header");

        let stack = imp.stack
            .borrow()
            .clone();
        self.build_disconnected_ui(&stack);
        self.build_empty_ui(&stack);
        self.build_idle_ui(&stack);
        self.build_copying_ui(&stack);
        self.build_success_ui(&stack);
        self.build_failed_ui(&stack);

        let content_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .width_request(800)
            .build();
        content_row.append(&stack);
        content_row.add_css_class("drive-widget-content");

        let disc_label = imp.disc_label
            .borrow()
            .clone();
        disc_label.set_halign(Align::Start);
        disc_label.set_hexpand(true);

        let path_label = imp.path_label
            .borrow()
            .clone();
        path_label.set_hexpand(false);

        let serial_number_label = imp.serial_number_label
            .borrow()
            .clone();
        serial_number_label.set_hexpand(false);

        let footer_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_bottom(4)
            .margin_end(8)
            .margin_start(8)
            .margin_top(4)
            .build();
        footer_row.append(&disc_label);
        footer_row.append(&path_label);
        footer_row.append(&serial_number_label);
        footer_row.add_css_class("drive-widget-footer");

        self.add_css_class("drive-widget");
        self.set_orientation(Orientation::Vertical);
        self.set_spacing(0);
        self.append(&header_row);
        self.append(&content_row);
        self.append(&footer_row);

        imp.clear_button.replace(clear_button);
        imp.copy_button.replace(copy_button);
        imp.reset_button.replace(reset_button);
        imp.cancel_button.replace(cancel_button);
    }

    /// Builds the view when the drive is not connected.
    ///
    /// # Args
    ///
    /// `stack`:  The stack widget that the view should be added to..
    fn build_disconnected_ui(&self, stack: &Stack) {
        let icon_box = self.build_icon_box(
            Some("fontawesome.v7.solid.plug-circle-minus-symbolic"),
            "disconnected"
        );

        let content = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .label("The drive is currently unavailable.")
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .build();

        let disconnected_view = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        disconnected_view.append(&icon_box);
        disconnected_view.append(&content);

        stack.add_named(&disconnected_view, Some("disconnected-view"));
        self.imp().disconnected_view.replace(disconnected_view);
    }

    /// Builds an icon box.
    ///
    /// The icon box is the icon that is displayed on the left side of the widget. It changes based
    /// on the current state of the drive.
    ///
    /// # Args
    ///
    /// `icon`:  If `Some`, the name of the icon that should be displayed. This is the file name
    /// of the SVG file without the path or extension. `None` if an icon should not be displayed.
    ///
    /// `class`:  The CSS class to assign the box.
    fn build_icon_box(&self, icon: Option<&str>, class: &str) -> Box {
        let icon_box = Box::builder()
            .height_request(128)
            .width_request(128)
            .orientation(Orientation::Vertical)
            .build();
        icon_box.add_css_class("drive-widget-icon-container");
        icon_box.add_css_class(class);

        if let Some(icon) = icon {
            let icon = Image::builder()
                .valign(Align::Center)
                .vexpand(true)
                .halign(Align::Center)
                .icon_name(icon)
                .pixel_size(100)
                .build();
            icon.add_css_class("drive-widget-icon");
            icon.add_css_class(class);
            icon_box.append(&icon)
        };

        icon_box
    }

    /// Builds the view used when the drive is empty.
    ///
    /// # Args
    ///
    /// `stack`:  The stack widget that the view should be added to..
    fn build_empty_ui(&self, stack: &Stack) {
        let icon_box = self.build_icon_box(None, "idle");

        let content = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .label("The drive is empty")
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .build();

        let empty_view = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        empty_view.append(&icon_box);
        empty_view.append(&content);

        stack.add_named(&empty_view, Some("empty-view"));
        self.imp().empty_view.replace(empty_view);
    }

    /// Builds the view used when a disc is inserted into the drive and the drive is waiting for a
    /// copy operation to be started.
    ///
    /// # Args
    ///
    /// `stack`:  The stack widget that the view should be added to..
    fn build_idle_ui(&self, stack: &Stack) {
        let icon_box = self.build_icon_box(
            Some("fontawesome.v7.solid.compact-disc-symbolic"),
            "idle"
        );

        let copy_form = CopyFormWidget::new();

        let idle_view = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        idle_view.append(&icon_box);
        idle_view.append(&copy_form);

        stack.add_named(&idle_view, Some("idle-view"));

        let imp = self.imp();
        imp.copy_form.replace(copy_form);
        imp.idle_view.replace(idle_view);
    }

    /// Builds the view used when a copy operation is in progress.
    ///
    /// # Args
    ///
    /// `stack`:  The stack widget that the view should be added to..
    fn build_copying_ui(&self, stack: &Stack) {
        let icon_box = self.build_icon_box(
            Some("fontawesome.v7.solid.compact-disc-symbolic"),
            "copying"
        );

        let stage_label = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .build();

        let elapsed_label = Label::builder()
            .build();

        let row_0 = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_bottom(12)
            .build();
        row_0.append(&stage_label);
        row_0.append(&elapsed_label);

        let task_label = Label::builder()
            .halign(Align::Start)
            .margin_bottom(4)
            .build();
        task_label.add_css_class("drive-widget-task-label");

        let task_progress = ProgressBar::builder()
            .margin_bottom(8)
            .build();

        let subtask_label = Label::builder()
            .halign(Align::Start)
            .margin_bottom(4)
            .build();
        subtask_label.add_css_class("drive-widget-task-label");

        let subtask_progress = ProgressBar::builder()
            .build();

        let content = Box::builder()
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .orientation(Orientation::Vertical)
            .build();
        content.append(&row_0);
        content.append(&task_label);
        content.append(&task_progress);
        content.append(&subtask_label);
        content.append(&subtask_progress);

        let copying_view = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        copying_view.append(&icon_box);
        copying_view.append(&content);

        stack.add_named(&copying_view, Some("copying-view"));

        let imp = self.imp();
        imp.copying_view.replace(copying_view);
        imp.stage_label.replace(stage_label);
        imp.elapsed_label.replace(elapsed_label);
        imp.task_label.replace(task_label);
        imp.task_progress.replace(task_progress);
        imp.subtask_label.replace(subtask_label);
        imp.subtask_progress.replace(subtask_progress);
    }

    /// Builds the view used when a copy operation has completed successfully.
    ///
    /// # Args
    ///
    /// `stack`:  The stack widget that the view should be added to..
    fn build_success_ui(&self, stack: &Stack) {
        let icon_box = self.build_icon_box(
            Some("fontawesome.v7.solid.circle-check-symbolic"),
            "success"
        );

        let content = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .label("The copy operation has completed successfully.")
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .build();

        let success_view = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        success_view.append(&icon_box);
        success_view.append(&content);

        stack.add_named(&success_view, Some("success-view"));
        self.imp().success_view.replace(success_view);
    }

    /// Builds the view used when a copy operation failed or was cancelled.
    ///
    /// # Args
    ///
    /// `stack`:  The stack widget that the view should be added to..
    fn build_failed_ui(&self, stack: &Stack) {
        let icon_box = self.build_icon_box(
            Some("fontawesome.v7.solid.exclamation-triangle"),
            "failed"
        );

        let notice = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .label("The copy operation failed to complete.")
            .build();

        let error_message = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .build();

        let error_message_box = Box::builder()
            .build();
        error_message_box.add_css_class("drive-widget-error-message");
        error_message_box.append(&error_message);

        let content = Box::builder()
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .orientation(Orientation::Vertical)
            .spacing(8)
            .valign(Align::Center)
            .vexpand(true)
            .build();
        content.append(&notice);
        content.append(&error_message_box);

        let failed_view = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        failed_view.append(&icon_box);
        failed_view.append(&content);

        stack.add_named(&failed_view, Some("failed-view"));
        let imp = self.imp();
        imp.failed_view.replace(failed_view);
        imp.error_message.replace(error_message);
    }

    /// Configures the signals and callbacks.
    ///
    /// Called by the implementation ([`imp::DriveWidget`]) when constructed.
    fn setup_callbacks(&self) {
        let imp = self.imp();

        let copy_form = imp.copy_form
            .borrow()
            .clone();
        imp.clear_button.borrow().connect_clicked(move |_| {
            copy_form.clear();
        });
    }

    /// Binds the widget to the provided optical drive object.
    ///
    /// # Args
    /// 
    /// `drive_object`:  The drive object to bind to.
    pub fn bind(&self, drive_object: &OpticalDriveObject) {
        let imp = self.imp();
  
        let mut bindings = imp.bindings.borrow_mut();
  
        let name_label = imp.name_label.borrow();
        let name_binding = drive_object
            .bind_property("name", &name_label.clone(), "label")
            .sync_create()
            .build();
        bindings.push(name_binding);
  
        let path_label = imp.path_label.borrow();
        let path_binding = drive_object
            .bind_property("path", &path_label.clone(), "label")
            .transform_to(|_, d: String| {
                Some(format!("[ {} ]", d).to_value())
            })
            .sync_create()
            .build();
        bindings.push(path_binding);
  
        let serial_number_label = imp.serial_number_label.borrow();
        let serial_number_binding = drive_object
            .bind_property("serial-number", &serial_number_label.clone(), "label")
            .transform_to(|_, serial_number: String| {
                Some(format!("[ {} ]", serial_number).to_value())
            })
            .sync_create()
            .build();
        bindings.push(serial_number_binding);
  
        let disc_label = imp.disc_label.borrow();
        let disc_binding = drive_object
            .bind_property("disc-label", &disc_label.clone(), "label")
            .transform_to(|_, disc_label: String| {
                if disc_label.is_empty() {
                    Some(String::from("No Disc"))
                } else {
                    Some(disc_label)
                }
            })
            .sync_create()
            .build();
        bindings.push(disc_binding);
  
        let stack = imp.stack.borrow();
        let stack_binding = drive_object
            .bind_property("drive-state", &stack.clone(), "visible-child-name")
            .transform_to(|_, state: OpticalDriveState| {
                let view_name = match state {
                    OpticalDriveState::Disconnected => String::from("disconnected-view"),
                    OpticalDriveState::Empty => String::from("empty-view"),
                    OpticalDriveState::Idle => String::from("idle-view"),
                    OpticalDriveState::Copying => String::from("copying-view"),
                    OpticalDriveState::Success => String::from("success-view"),
                    OpticalDriveState::Failed => String::from("failed-view"),
                };
                Some(view_name)
            })
            .sync_create()
            .build();
        bindings.push(stack_binding);
  
        let error_message = imp.error_message.borrow();
        let error_binding = drive_object
            .bind_property("error-message", &error_message.clone(), "label")
            .sync_create()
            .build();
        bindings.push(error_binding);

        let stage_label = imp.stage_label.borrow();
        let stage_binding = drive_object
            .bind_property("stage", &stage_label.clone(), "label")
            .sync_create()
            .build();
        bindings.push(stage_binding);
  
        let elapsed_label = imp.elapsed_label.borrow();
        let elapsed_binding = drive_object
            .bind_property("elapsed-time", &elapsed_label.clone(), "label")
            .sync_create()
            .build();
        bindings.push(elapsed_binding);
  
        let task_label = imp.task_label.borrow();
        let task_binding = drive_object
            .bind_property("task", &task_label.clone(), "label")
            .sync_create()
            .build();
        bindings.push(task_binding);
  
        let task_progress = imp.task_progress.borrow();
        let task_binding = drive_object
            .bind_property("task-progress", &task_progress.clone(), "fraction")
            .sync_create()
            .build();
        bindings.push(task_binding);
  
        let subtask_label = imp.subtask_label.borrow();
        let subtask_binding = drive_object
            .bind_property("subtask", &subtask_label.clone(), "label")
            .sync_create()
            .build();
        bindings.push(subtask_binding);
  
        let subtask_progress = imp.subtask_progress.borrow();
        let subtask_binding = drive_object
            .bind_property("subtask-progress", &subtask_progress.clone(), "fraction")
            .sync_create()
            .build();
        bindings.push(subtask_binding);
  
        let clear_button = imp.clear_button.borrow();
        let clear_visibility_binding = drive_object
            .bind_property("drive-state", &clear_button.clone(), "visible")
            .transform_to(|_, state: OpticalDriveState| {
                Some(state == OpticalDriveState::Idle)
            })
            .sync_create()
            .build();
        bindings.push(clear_visibility_binding);
  
        let copy_button = imp.copy_button.borrow();
        let copy_visibility_binding = drive_object
            .bind_property("drive-state", &copy_button.clone(), "visible")
            .transform_to(|_, state: OpticalDriveState| {
                Some(state == OpticalDriveState::Idle)
            })
            .sync_create()
            .build();
        bindings.push(copy_visibility_binding);
  
        if let Some(form_data) = task::block_on(drive_object.read_form_data()) {
            imp.copy_form.borrow().set_form_data(&form_data);
        };

        let copy_form = imp.copy_form
            .borrow()
            .clone();
        let drive = drive_object
            .clone();
        copy_button.connect_clicked(move |_| {
            if !copy_form.validate() {
                tracing::debug!(form=%copy_form, "copy form invalid");
                return;
            }
  
            let copy_parameters = copy_form.get_copy_parameters();
  
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.copy_disc(copy_parameters).await;
                }
            ));
        });

        let reset_button = imp.reset_button.borrow();
        let reset_visibility_binding = drive_object
            .bind_property("drive-state", &reset_button.clone(), "visible")
            .transform_to(|_, state: OpticalDriveState| {
                Some(state == OpticalDriveState::Success || state == OpticalDriveState::Failed)
            })
            .sync_create()
            .build();
        bindings.push(reset_visibility_binding);

        let drive = drive_object
            .clone();
        imp.reset_button.borrow().connect_clicked(move |_| {
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move { 
                    drive.reset().await; 
                }
            ));
        });

        let cancel_button = imp.cancel_button.borrow();
        let cancel_visibility_binding = drive_object
            .bind_property("drive-state", &cancel_button.clone(), "visible")
            .transform_to(|_, state: OpticalDriveState| {
                Some(state == OpticalDriveState::Copying)
            })
            .sync_create()
            .build();
        bindings.push(cancel_visibility_binding);
  
        let drive = drive_object
            .clone();
        imp.cancel_button.borrow().connect_clicked(move |_| {
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.cancel_copy_disc().await;
                }
            ));
        });

        let copy_form = imp.copy_form.borrow();

        let drive = drive_object
            .clone();
        copy_form.connect_media_type_changed(move |media_type| {
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    let data = FormDataUpdate::media_type(media_type.as_str().to_owned());
                    drive.save_form_data(data).await;
                }
            ));
        });

        let drive = drive_object
            .clone();
        copy_form.connect_title_changed(move |title| {
            let title = title.to_owned();
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.save_form_data(FormDataUpdate::title(title)).await;
                }
            ));
        });

        let drive = drive_object
            .clone();
        copy_form.connect_year_changed(move |year| {
            let year = year.to_owned();
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.save_form_data(FormDataUpdate::year(year)).await;
                }
            ));
        });
  
        let drive = drive_object
            .clone();
        copy_form.connect_disc_number_changed(move |disc_number| {
            let disc_number = disc_number.to_owned();
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.save_form_data(
                        FormDataUpdate::disc_number(disc_number),
                    ).await;
                }
            ));
        });
  
        let drive = drive_object
            .clone();
        copy_form.connect_season_number_changed(move |season_number| {
            let season_number = season_number.to_owned();
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.save_form_data(
                        FormDataUpdate::season_number(season_number)
                    ).await;
                }
            ));
        });
  
        let drive = drive_object
            .clone();
        copy_form.connect_location_changed(move |location| {
            let location = location.to_owned();
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.save_form_data(
                        FormDataUpdate::storage_location(location)
                    ).await;
                }
            ));
        });
  
        let drive = drive_object
            .clone();
        copy_form.connect_memo_changed(move |memo| {
            let memo = memo.to_owned();
            glib::spawn_future_local(glib::clone!(
                #[weak]
                drive,
                async move {
                    drive.save_form_data(FormDataUpdate::memo(memo)).await;
                }
            ));
        });
    }

    /// Unbinds the drive widget from the optical drive object which was bound when
    /// [`DriveWidget::bind`] was called.
    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}

mod imp {
    //! Implementation for the optical drive widget.

    use std::cell::RefCell;

    use gtk::{Box, Label, ProgressBar, Stack};
    use gtk::glib::{self, Binding};
    use gtk::subclass::prelude::*;

    use crate::ui::widget::{CopyFormWidget, IconButton};

    /// Implementation for [`super::DriveWidget`].
    #[derive(Default)]
    pub struct DriveWidget {
        /// Label widget for displaying the name of the drive.
        pub(super) name_label: RefCell<Label>,

        /// Button used to clear the form.
        pub(super) clear_button: RefCell<IconButton>,

        /// Button used to start a copy operation.
        pub(super) copy_button: RefCell<IconButton>,

        /// Button used to reset the drive state after a successful or failed copy operation.
        pub(super) reset_button: RefCell<IconButton>,

        /// Button used stop an in-progress copy operation.
        pub(super) cancel_button: RefCell<IconButton>,

        /// Label widget for displaying the device path of the drive.
        pub(super) path_label: RefCell<Label>,

        /// Label widget for displaying the serial number of the drive.
        pub(super) serial_number_label: RefCell<Label>,

        /// Label widget for displaying the disc label of the disc inserted into the drive.
        pub(super) disc_label: RefCell<Label>,

        /// Stack used to control which view is displayed.
        pub(super) stack: RefCell<Stack>,

        /// View when the drive is in the disconnected state.
        pub(super) disconnected_view: RefCell<Box>,

        /// View when the drive is in the idle state, but does not have a disc.
        pub(super) empty_view: RefCell<Box>,

        /// View when the drive is in the idle state.
        pub(super) idle_view: RefCell<Box>,

        /// Form used to enter copy parameters.
        pub(super) copy_form: RefCell<CopyFormWidget>,

        /// View when the drive is performing a copy operation.
        pub(super) copying_view: RefCell<Box>,

        /// Displays the current stage of a copy operation.
        pub(super) stage_label: RefCell<Label>,

        /// Displays the elapsed time for the copy operation.
        pub(super) elapsed_label: RefCell<Label>,

        /// Displays the name of the current task.
        pub(super) task_label: RefCell<Label>,

        /// Displays the progress of the current task.
        pub(super) task_progress: RefCell<ProgressBar>,

        /// Displays the name of the current subtask.
        pub(super) subtask_label: RefCell<Label>,

        /// Displays the progress of the current subtask.
        pub(super) subtask_progress: RefCell<ProgressBar>,

        /// View when the drive successfully completed a copy operation.
        pub(super) success_view: RefCell<Box>,

        /// View when the drive failed to complete a copy operation (or was cancelled).
        pub(super) failed_view: RefCell<Box>,

        /// The error message displayed in the failed view.
        pub(super) error_message: RefCell<Label>,

        /// The widget's bindings.
        ///
        /// This is populated when the widget is bound to a optical drive object and cleared when
        /// unbound. See ([`super::DriveWidget::bind`]) and ([`super::DriveWidget::unbind`]).
        pub(super) bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DriveWidget {
        const NAME: &'static str = "ArtieDriveWidget";
        type Type = super::DriveWidget;
        type ParentType = Box;
    }

    impl ObjectImpl for DriveWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
            obj.setup_callbacks();
        }
    }

    impl WidgetImpl for DriveWidget {}

    impl BoxImpl for DriveWidget {}
}

#[cfg(test)]
mod tests {
    // TODO
}
