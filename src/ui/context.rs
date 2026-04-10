// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the application's context object.

use std::path::PathBuf;

use gtk::gio::ListStore;
use gtk::glib;
use gtk::glib::Object;
use gtk::subclass::prelude::*;

use crate::{Mode, Result};
use crate::error::Error;
use crate::db;
use crate::drive;
use crate::fs::FileSystem;
use crate::net::client;
use crate::net::client::ClientManagerHandle;
use crate::settings::Settings;

glib::wrapper! {
    /// Application context object.
    ///
    /// This context object contains the application state data that is used throughout the
    /// application.
    pub struct ContextObject(ObjectSubclass<imp::ContextObject>);
}

impl ContextObject {
    /// Creates a [`ContextObjectBuilder`] instance for building the context.
    pub fn builder() -> ContextObjectBuilder {
        ContextObjectBuilder {
            mode: Mode::default(),
        }
    }

    /// Returns the drive data store.
    ///
    /// The drive data store contains the optical drive data as a list of
    /// [`crate::drive::glib::optical_drive::OpticalDriveObject`] objects.
    pub fn drives_store(&self) -> Option<ListStore> {
        self.imp().drive_store
            .borrow()
            .clone()
    }

    /// Returns true if the application is running in worker mode.
    pub fn is_worker(&self) -> bool {
        self.imp().mode.get() != Mode::Control
    }

    /// Creates a new [`ContextObject`] instance.
    fn new() -> Self {
        Object::builder().build()
    }

//  /// Saves the settings to the config.
//  ///
//  /// This will create the file if it does not exist or overwrite the file if it does. See
//  /// [`get_config_path`] for more information on how the path is determined.
//  ///
//  /// # Errors
//  ///
//  /// - [`Error::FileIo`] if the file cannot be written to, or
//  /// - [`Error::Serialization`] if the settings could not be serialized.
//  pub fn save_settings(&self) -> Result<()> {
//      let path = get_config_path();
//      self.settings.save(&path)
//  }

}

/// Builder used to construct the application context.
pub struct ContextObjectBuilder {
    /// The mode the application is being run in.
    mode: Mode,
}

impl ContextObjectBuilder {
    /// Sets the application mode.
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Builds the context object.
    ///
    /// # Errors
    ///
    /// - [`crate::Error::FileIo`] if the config file cannot be read. This will also be raised when
    ///   an I/O occurs while creating the data directories.
    /// - [`crate::Error::FileNotFound`] if the config file cannot be found.
    /// - [`crate::Error::Serialization`] if the config file's content cannot be deserialized.
    /// - See [`drive::init`] for potential errors that can occur searching for optical drives
    ///   and initializing their actor instance.
    pub fn build(self) -> Result<ContextObject> {
        let context = ContextObject::new();
        let imp = context.imp();

        // TODO: Need to safely handle the config file not existing. Maybe create a default version
        //       if it cannot be found.
        let path = get_config_path();
        if !path.is_file() {
            return Err(Error::FileNotFound { path });
        }

        let settings = Settings::from_file(&path)?;

        let fs = FileSystem::new(&settings.fs);
        fs.make_dirs()?;

        let db = db::init(&fs)?;

        let client_mgr = client::create_client_manager();

        let optical_drives = drive::init(fs, db)?;

        let drive_store = ListStore::from_iter(optical_drives);

        imp.drive_store.replace(Some(drive_store));
        imp.client_mgr.replace(Some(client_mgr));
        imp.mode.set(self.mode);

        Ok(context)
    }
}

/// Get the path to the application's config file.
///
/// TODO: This currently just returns a hard-coded path for the purposes of development. It will
///       need to be updated to look at an environment variable first, then fallback to a standard
///       location based on the OS (e.g. ~/.config/artie or %AppData%/artie).
fn get_config_path() -> PathBuf {
    PathBuf::from("artie.toml")
}

// TODO: Tests:

mod imp {
    //! Object implementation.

    use std::cell::{Cell, RefCell};

    use gtk::glib;
    use gtk::gio::ListStore;
    use gtk::subclass::prelude::*;

    use crate::Mode;
    use crate::net::client::ClientManagerHandle;

    /// Implementation for [`super::ContextObject`].
    #[derive(Default)]
    pub struct ContextObject {
        /// Store containing the optical drive data.
        ///
        /// Contains [`crate::drive::glib::optical_drive::OpticalDriveObject`] objects.
        pub(super) drive_store: RefCell<Option<ListStore>>,

        /// Interface for communicating with the actor responsible for managing client connections
        /// to the worker nodes.
        pub(super) client_mgr: RefCell<Option<ClientManagerHandle>>,

        /// The current application mode.
        pub(super) mode: Cell<Mode>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContextObject {
        const NAME: &'static str = "ContextObject";
        type Type = super::ContextObject;
    }

    impl ObjectImpl for ContextObject {
    }
}
