// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use std::time::Duration;

use tokio::time;

use crate::bus;
use crate::drive;

/// Task for periodically checking the status of the drive.
///
/// This will not exit until the receiving end of the actor's channel is closed.
pub async fn monitor_drive(bus: bus::Handle, serial_number: String) {
    // TODO: Should there be some sort of max error count, increase the time between checks on
    //       error, or both.
    loop {
        match drive::get_optical_drive(&serial_number) {
            Ok(info) => {
                let _ = drive::update_from_os(&bus, &serial_number, info).await;
            },
            Err(error) => {
                tracing::error!(sn=serial_number, ?error, "failed to get drive info from OS");
            }
        }

        time::sleep(Duration::from_secs(1)).await;
    }
}
