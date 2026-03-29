// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use chrono::Utc;

use rusqlite::Connection;

use tokio::sync::mpsc;

use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;

use makemkv;
use makemkv::CommandOutput;

use crate::Error;
use crate::db;
use crate::db::Database;
use crate::fs::FileSystem;
use crate::drive::{DiscState, OpticalDriveState, OpticalDrive};
use crate::drive::actor::handle::DriveActorHandle;
use crate::library;
use crate::models::{CopyOperation, CopyParamaters, OperationState, Reference};

/// Copies the disc in the optical drive.
pub async fn copy_disc(
    drive: OpticalDrive,
    copy_parameters: CopyParamaters,
    fs: FileSystem,
    db: Database,
    actor: DriveActorHandle,
    ct: CancellationToken,
) {
    tracing::info!(sn=drive.serial_number, "starting copy operation");

    let mut conn = match db.connect() {
        Ok(db) => db,
        Err(error) => {
            tracing::error!(sn=drive.serial_number, ?error, "database connection failed");
            operation_failed(
                &actor,
                &drive.serial_number,
                None,
                ErrorMessage::ConnectFailed(error),
            ).await;
            return;
        }
    };

    let DiscState::Inserted { label: _disc_label, uuid: disc_uuid } = drive.disc else {
        tracing::error!("cannot copy from empty drive");
        operation_failed(
            &actor, 
            &drive.serial_number,
            None,
            ErrorMessage::InvalidDiscState,
        ).await;
        return;
    };

    let db_drive = match db::optical_drive::get_or_create(&conn, &drive.serial_number) {
        Ok(drive) => drive,
        Err(error) => {
            tracing::error!(
                sn=drive.serial_number,
                ?error, 
                "failed to get/create optical drive db record"
            );
            operation_failed(
                &actor,
                &drive.serial_number,
                None,
                ErrorMessage::DbOpOpticalDriveFailed(error),
            ).await;
            return;
        }
    };

    // Don't expect a computer's hostname to contain invalid unicode characters. If it does, then
    // something likely went very wrong with fetching the hostname to the point where we would
    // prefer an application crash anyways.
    // 
    // TODO: When worker nodes are added, the following will need to be updated.
    // 
    let hostname = gethostname::gethostname()
        .into_string()
        .unwrap();

    let host = match db::host::get_or_create(&conn, &hostname) {
        Ok(host) => host,
        Err(error) => {
            tracing::error!(sn=drive.serial_number, ?error, "failed to get/create host db record");
            operation_failed(
                &actor,
                &drive.serial_number,
                None,
                ErrorMessage::DbOpHostFailed(error),
            ).await;
            return;
        }
    };

    let mut copy_operation = CopyOperation {
        started: Utc::now(),
        media_type: copy_parameters.media_type,
        title: copy_parameters.title,
        year: copy_parameters.release_year,
        disc: copy_parameters.disc_number,
        disc_uuid: disc_uuid.clone(),
        season: copy_parameters.season_number,
        location: copy_parameters.location,
        memo: copy_parameters.memo,
        drive: Reference {
            id: db_drive.id,
            value: None
        },
        host: Reference {
            id: host.id,
            value: None
        },
        ..CopyOperation::default()
    };

    if let Err(error) = db::copy_operation::create(&conn, &mut copy_operation) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to get/create host db record");
        operation_failed(
            &actor,
            &drive.serial_number,
            None,
            ErrorMessage::DbOpCopyOperationCreateFailed(error),
        ).await;
        return;
    };

    // Don't check for cancellation until now because we want there to be a database entry.
    if ct.is_cancelled() {
        tracing::info!(sn=drive.serial_number, "copy operation cancelled");
        operation_canceled(&actor, &drive.serial_number, conn, copy_operation).await;
        return;
    }

    if let Err(error) = db::copy_operation::set_state(
        &conn,
        &mut copy_operation,
        OperationState::Running
    ) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to set running state in db");
        operation_failed(
            &actor,
            &drive.serial_number,
            None,
            ErrorMessage::DbOpSetStateRunning(error),
        ).await;
        return;
    }

    let output_dir = fs.inbox_folder(&copy_operation);

    // The folder name should almost guaranteed to be unique. Even so, if it does, exit because
    // something has gone wrong so we don't override stuff we don't intend to.
    if output_dir.exists() {
        tracing::error!(sn=drive.serial_number, ?output_dir, "output directory already exists");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::OutputDirExists,
        ).await;
        return;
    }

    if let Err(error) = std::fs::create_dir(&output_dir) {
        tracing::error!(
            sn=drive.serial_number,
            ?output_dir,
            ?error,
            "failed to create output directory"
        );
        let error = Error::FileIo { 
            path: output_dir, 
            error 
        };
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::OutputDirCreateFailed(error),
        ).await;
        return;
    }

    tracing::info!(sn=drive.serial_number, path=?output_dir, "created inbox folder");

    let (tx, mut rx) = mpsc::unbounded_channel::<CommandOutput>();
    let device_path = drive.path.clone();
    let tx_clone = tx.clone();
    let log_path = fs.mkv_info_log_file(&copy_operation);
    let ct_clone = ct.clone();
    let handle = tokio::spawn(async move {
        makemkv::get_disc_info(&device_path, &tx_clone, &log_path, &ct_clone).await
    }).with_cancellation_token(&ct);

    // Must drop the original sender to avoid blocking indefinitely.
    drop(tx);

    while let Some(data) = rx.recv().await {
        match data {
            CommandOutput::Message(_message) => {
                // TODO
            },
            CommandOutput::Progress(progress) => {
                let state = OpticalDriveState::Copying {
                    stage: String::from("Gathering Disc Info"),
                    task: progress.op.clone(),
                    task_progress: (progress.op_prog as f32) / 100.0,
                    subtask: progress.subop.clone(),
                    subtask_progress: (progress.subop_prog as f32) / 100.0,
                    elapsed_time: Duration::ZERO,
                };
                send_state(&actor, state).await;
            },
            CommandOutput::Error(_error) => {
                // TODO
            },
        }
    }

    if ct.is_cancelled() {
        tracing::info!(sn=drive.serial_number, "copy operation cancelled");
        operation_canceled(&actor, &drive.serial_number, conn, copy_operation).await;
        return;
    }


    let result = handle.await;

    // NOTE: If we make it this far, result should not be None. If its None, its because the token
    //       is cancelled which means we would have exited above. Leaving this hear just in case
    //       there are other conditions then the token being cancelled that can result in None that
    //       aren't documented or something changes in the future.
    let Some(result) = result else {
        tracing::error!(sn=drive.serial_number, "attempted to run info task with cancelled token");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::InfoCommandHandleAwait,
        ).await;
        return;
    };

    let result = match result {
        Ok(result) => result,
        Err(error) => {
            tracing::error!(sn=drive.serial_number, ?error, "failed to join info task");
            operation_failed(
                &actor,
                &drive.serial_number,
                Some((conn, copy_operation)),
                ErrorMessage::InfoCommandJoinError,
            ).await;
            return;
        },
    };

    let (disc_info, log_text) = match result {
        Ok(output) => (output.disc_info, output.log),
        Err(error) => {
            tracing::error!(sn=drive.serial_number, ?error, "disc info command failed");
            let error = Error::MakeMKV { error };
            operation_failed(
                &actor,
                &drive.serial_number,
                Some((conn, copy_operation)),
                ErrorMessage::MkvInfoCommandFailed(error),
            ).await;
            return;
        },
    };

    let path = fs.disc_info_file(&copy_operation);
    if let Err(error) = disc_info.save(&path) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to save disc info to disc");
        let error = Error::MakeMKV { error };
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::DiscInfoSaveFailed(error),
        ).await;
        return;
    }

    if let Err(error) = db::copy_operation::set_metadata(&conn, &mut copy_operation, &disc_info) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to write disc info to db");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::DbOpSetMetadataFailed(error),
        ).await;
        return;
    }

    if let Err(error) = db::copy_operation::set_info_log(&conn, &mut copy_operation, &log_text) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to write info log to db");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::DbOpSetInfoLogFailed(error),
        ).await;
        return;
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<CommandOutput>();
    let device_path = drive.path.clone();
    let tx_clone = tx.clone();
    let log_path = fs.mkv_copy_log_file(&copy_operation);
    let ct_clone = ct.clone();
    let handle = tokio::spawn(async move {
        makemkv::copy_disc(&device_path, &output_dir, &tx_clone, &log_path, &ct_clone).await
    }).with_cancellation_token(&ct);

    // Must drop the original sender to avoid blocking indefinitely.
    drop(tx);

    while let Some(data) = rx.recv().await {
        match data {
            CommandOutput::Message(_message) => {
                // TODO
            },
            CommandOutput::Progress(progress) => {
                let state = OpticalDriveState::Copying {
                    stage: String::from("Copying Disc"),
                    task: progress.op.clone(),
                    task_progress: (progress.op_prog as f32) / 100.0,
                    subtask: progress.subop.clone(),
                    subtask_progress: (progress.subop_prog as f32) / 100.0,
                    elapsed_time: Duration::ZERO,
                };
                send_state(&actor, state).await;
            },
            CommandOutput::Error(_error) => {
                // TODO
            },
        }
    }

    if ct.is_cancelled() {
        tracing::info!(sn=drive.serial_number, "copy operation cancelled");
        operation_canceled(&actor, &drive.serial_number, conn, copy_operation).await;
        return;
    }

    let result = handle.await;

    // NOTE: If we make it this far, result should not be None. If its None, its because the token
    //       is cancelled which means we would have exited above. Leaving this hear just in case
    //       there are other conditions then the token being cancelled that can result in None that
    //       aren't documented or something changes in the future.
    let Some(result) = result else {
        tracing::error!(sn=drive.serial_number, "attempted to run copy task with cancelled token");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::CopyCommandHandleAwait,
        ).await;
        return;
    };

    let result = match result {
        Ok(result) => result,
        Err(error) => {
            tracing::error!(sn=drive.serial_number, ?error, "failed to join copy task");
            operation_failed(
                &actor,
                &drive.serial_number,
                Some((conn, copy_operation)),
                ErrorMessage::CopyCommandJoinError,
            ).await;
            return;
        },
    };

    let log_text = match result {
        Ok(output) => output.log,
        Err(error) => {
            tracing::error!(sn=drive.serial_number, ?error, "disc copy command failed");
            let error = Error::MakeMKV { error };
            operation_failed(
                &actor,
                &drive.serial_number,
                Some((conn, copy_operation)),
                ErrorMessage::MkvCopyCommandFailed(error),
            ).await;
            return;
        },
    };

    if let Err(error) = db::copy_operation::set_copy_log(&conn, &mut copy_operation, &log_text) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to write copy log to db");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::DbOpSetCopyLogFailed(error),
        ).await;
        return;
    }

    if let Err(error) = library::process_copy_operation(
        &fs,
        &drive.serial_number,
        &mut conn,
        &disc_info,
        &copy_operation
    ) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to generate videos and titles");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::CreateVideosAndTitlesFailed(error),
        ).await;
        return;
    }

    if let Err(error) = db::copy_operation::set_state(
        &conn,
        &mut copy_operation,
        OperationState::Completed
    ) {
        tracing::error!(sn=drive.serial_number, ?error, "failed to set state to completed");
        operation_failed(
            &actor,
            &drive.serial_number,
            Some((conn, copy_operation)),
            ErrorMessage::DbOpSetCopyLogFailed(error),
        ).await;
        return;
    }
    
    send_state(&actor, OpticalDriveState::Success).await;

    tracing::info!(sn=drive.serial_number, "copy operation completed successfully");
}

/// Specifies the various error messages that can occur during a copy operation.
#[allow(dead_code)]
#[derive(Debug)]
enum ErrorMessage {
    ConnectFailed(Error),
    CopyCommandHandleAwait,
    CopyCommandJoinError,
    CreateVideosAndTitlesFailed(Error),
    DbOpCopyOperationCreateFailed(Error),
    DbOpHostFailed(Error),
    DbOpOpticalDriveFailed(Error),
    DbOpSetCopyLogFailed(Error),
    DbOpSetInfoLogFailed(Error),
    DbOpSetMetadataFailed(Error),
    DbOpSetStateCompleted(Error),
    DbOpSetStateRunning(Error),
    DiscInfoSaveFailed(Error),
    DiscInfoSerializationError(Error),
    InfoCommandHandleAwait,
    InfoCommandJoinError,
    InvalidDiscState,
    MkvCopyCommandFailed(Error),
    MkvInfoCommandFailed(Error),
    OutputDirCreateFailed(Error),
    OutputDirExists,
}

impl ErrorMessage {
    /// Creates the error message for the user.
    fn user_message(&self) -> String {
        match self {
            ErrorMessage::ConnectFailed(_) => { 
                String::from("Database connection failed.")
            },
            ErrorMessage::CopyCommandHandleAwait => {
                String::from("System Error (copy-await).")
            },
            ErrorMessage::CopyCommandJoinError => {
                String::from("System Error (copy-join).")
            },
            ErrorMessage::CreateVideosAndTitlesFailed(_) => {
                String::from("Database operation failed: Failed to create title/video data")
            },
            ErrorMessage::DbOpCopyOperationCreateFailed(_) => {
                String::from("Database operation failed: Failed to create copy operation record.")
            },
            ErrorMessage::DbOpHostFailed(_) => {
                String::from("Database operation failed: Failed to get/create host record.")
            },
            ErrorMessage::DbOpOpticalDriveFailed(_) => {
                String::from("Database operation failed: Failed to get/create drive record.")
            },
            ErrorMessage::DbOpSetCopyLogFailed(_) => {
                String::from("Database operation failed: Failed to set copy command log.")
            },
            ErrorMessage::DbOpSetInfoLogFailed(_) => {
                String::from("Database operation failed: Failed to set info command log.")
            },
            ErrorMessage::DbOpSetMetadataFailed(_) => {
                String::from("Database operation failed: Failed to set metadata.")
            },
            ErrorMessage::DbOpSetStateCompleted(_) => {
                String::from("Database operation failed: Failed to update state to completed.")
            },
            ErrorMessage::DbOpSetStateRunning(_) => {
                String::from("Database operation failed: Failed to update state to running.")
            },
            ErrorMessage::DiscInfoSaveFailed(_) => {
                String::from("Failed to save disc information.")
            },
            ErrorMessage::DiscInfoSerializationError(_) => {
                String::from("Failed to serialize disc information.")
            },
            ErrorMessage::InfoCommandHandleAwait => {
                String::from("System Error (info-await).")
            },
            ErrorMessage::InfoCommandJoinError => {
                String::from("System Error (info-join).")
            },
            ErrorMessage::InvalidDiscState => {
                String::from("Cannot copy from empty drive.")
            },
            ErrorMessage::MkvCopyCommandFailed(_) => {
                String::from("Copying disc failed.")
            },
            ErrorMessage::MkvInfoCommandFailed(_) => {
                String::from("Failed to get disc information.")
            },
            ErrorMessage::OutputDirCreateFailed(_) => {
                String::from("Failed to create output directory.")
            },
            ErrorMessage::OutputDirExists => {
                String::from("Output directory already exists.")
            },
        }
    }

    /// Creates the error message for the database.
    fn database_message(&self) -> String {
        format!("{:?}", self)
    }
}

/// Updates the drive actor state to failed with a message indicating operation was cancelled.
async fn operation_canceled(
    actor: &DriveActorHandle,
    serial_number: &str,
    conn: Connection,
    copy_operation: CopyOperation,
) {
    let state = OpticalDriveState::Failed {
        error: String::from("Copy operation was cancelled."),
    };

    let mut copy_operation = copy_operation;

    if let Err(error) = db::copy_operation::set_state(
        &conn,
        &mut copy_operation,
        OperationState::Cancelled
    ) {
        tracing::info!(sn=serial_number, ?error, "failed to set cancelled state in database");
    }

    send_state(actor, state).await;
}

/// Updates the drive actor state to failed with the provided message.
async fn operation_failed(
    actor: &DriveActorHandle,
    serial_number: &str,
    data: Option<(Connection, CopyOperation)>,
    msg: ErrorMessage
) {
    let state = OpticalDriveState::Failed {
        error: msg.user_message(),
    };

    let operation_state = OperationState::Failed { 
        reason: msg.database_message(),
    };

    if let Some((conn, mut copy_operation)) = data
        && let Err(error) = db::copy_operation::set_state(
            &conn,
            &mut copy_operation,
            operation_state
        ) {
            tracing::info!(sn=serial_number, ?error, "failed to set failed state in database");
        }

    send_state(actor, state).await;
}

/// Updates the actor state.
async fn send_state(actor: &DriveActorHandle, state: OpticalDriveState) {
    actor.update_optical_drive_state(state).await.unwrap(); // FIXME
}

