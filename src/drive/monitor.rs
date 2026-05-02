// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles monitoring the optical drive state.
//!
//! [`monitor_drives`] is a task that will periodically get the status of the optical drives
//! attached to the local host from the OS and send the information to the drive actor.

use std::time::Duration;

use tokio::sync::oneshot;
use tokio::time;

use crate::Mode;
use crate::bus;
use crate::drive::{self, DriveRequest, ManagerRequest, Message};

/// Task for periodically checking the status of the drive.
///
/// # Args
///
/// `bus`:  Handle used to send requests to actors via the message bus.
///
/// `mode`:  The mode the application is running as.
pub async fn monitor_drives(bus: bus::Handle, mode: Mode) {
    loop {
        let drives = match drive::get_optical_drives() {
            Ok(drives) => drives,
            Err(error) => {
                tracing::error!(?error, "failed to get drive info from OS");
                break;
            }
        };

        // TODO: This is a temporary hack for testing multi-node on a single system. 
        if mode == Mode::Worker {
            for drive in drives {
                let (tx, rx) = oneshot::channel();
                let msg = Message::Drive {
                    serial_number: drive.serial_number.clone(),
                    request: DriveRequest::UpdateFromOs { drive, response: tx },
                };
                if let Err(error) = bus.send(msg).await {
                    tracing::error!(?error, "update drive status request failed");
                    continue;
                }
                if let Err(error) = rx.await {
                    tracing::error!(?error, "update drive status response failed");
                    continue;
                }
            }
        }

        if mode == Mode::Control {
            run_health_check(&bus).await;
        }

        // Only need to run occasionally.
        time::sleep(Duration::from_secs(1)).await;
    }
}

/// Run a health check on each drive which will check if the drive hasn't gotten an update in a
/// while.
///
/// # Args
///
/// `bus`:  Handle used to send requests to actors via the message bus.
async fn run_health_check(bus: &bus::Handle) {
    let (tx, rx) = oneshot::channel();
    let msg = Message::Manager {
        request: ManagerRequest::CheckDriveStatus { response: tx },
    };
    if let Err(error) = bus.send(msg).await {
        tracing::error!(?error, "drive check request failed");
        return;
    } 
    if let Err(error) = rx.await {
        tracing::error!(?error, "drive check response failed");
        return;
    }
}
