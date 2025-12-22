// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Linux implementation for OS specific code for interfacing with the optical drives.

use std::process::Command;

use serde::Deserialize;

use crate::{Error, Result};
use crate::error;

use super::{DiscState, OsOpticalDrive};

/// Represents the information returned by the `lsblk` command for an individual
/// block device.
#[derive(Debug, Deserialize)]
struct BlockDevice {
    /// The type of block device.
    ///
    /// The value for optical drives will be "rom".
    #[serde(rename = "type")]
    device_type: String,

    /// The block device's name (e.g. '/dev/sr0')
    name: String,

    /// The block device's serial number.
    ///
    /// This will always have a value for optical drives, but may not have a
    /// value for other block device types. For example, loopback devices will
    /// have a null serial number.
    #[serde(rename = "serial")]
    serial_number: Option<String>,

    /// The block device's label.
    ///
    /// For optical drives, this will only have a value when there is a disc
    /// inserted into the drive. The value will be based off the disc inserted
    /// into the drive.
    label: Option<String>,

    /// The block device's uuid.
    ///
    /// For optical drives, this will only have a value when there is a disc
    /// inserted into the drive. The value will be based off the disc inserted
    /// into the drive.
    uuid: Option<String>,
}

impl BlockDevice {
    /// Returns `true` if the block device represents an optical drive block device.
    fn is_optical_drive(&self) -> bool {
        self.device_type.eq_ignore_ascii_case("rom") && self.serial_number.is_some()
    }

    /// Returns `true` if the block device has serial number `serial_number`.
    fn has_serial_number(&self, serial_number: &str) -> bool {
        if let Some(ref sn) = self.serial_number {
            sn.eq_ignore_ascii_case(serial_number)
        } else {
            false
        }
    }

    /// Converts the block device to an [`OpticalDrive`] instance.
    ///
    /// Panics if called on a block device that is not a valid optical drive
    /// block device. Use [`BlockDevice::is_optical_drive`] to check if the
    /// device is a valid optical drive.
    fn to_optical_drive(&self) -> OsOpticalDrive {
        let Some(sn) = &self.serial_number else {
            panic!("Block device is not a valid optical drive.");
        };

        let mut drive = OsOpticalDrive {
            path: self.name.clone(),
            serial_number: sn.clone(),
            disc: DiscState::None,
        };

        if let Some(label) = &self.label {
            if let Some(uuid) = &self.uuid {
                drive.disc = DiscState::Inserted {
                    label: label.clone(),
                    uuid: uuid.clone(),
                }
            } else {
                drive.disc = DiscState::Inserted {
                    label: label.clone(),
                    uuid: String::from(""),
                }
            }
        }

        drive
    }
}

/// Represents the information returned by the `lsblk` command.
#[derive(Debug, Deserialize)]
struct BlockDeviceData {
    /// List of block devices.
    ///
    /// This will include all block devices, not just optical drives since there
    /// doesn't appear to be a way to construct the command in a way to only
    /// return optical drives.
    #[serde(rename = "blockdevices")]
    block_devices: Vec<BlockDevice>,
}

/// Runs the `lsblk` command and return the output which is a JSON string that
/// can be deserialized into a [`BlockDeviceData`] instance. Returns an error if
/// the command fails to run or exits with an error status code.
fn run_lsblk_command() -> Result<String> {
    let mut command = Command::new("lsblk");
    command.arg("--json");
    command.arg("--list");
    command.arg("--paths");
    command.arg("--output");
    command.arg("NAME,SERIAL,LABEL,TYPE,UUID");

    let output = command.output().map_err(|e| error::command_io(&command, e))?;

    if !output.status.success() {
        return Err(error::command_exit(&command, &output));
    }

    String::from_utf8(output.stdout).map_err(|e| Error::ConversionError { error: e })
}

// NOTE: The internal implementation allows for the bulk of the function to be
//       tested without having to make an actual call to the OS.

/// Internal implementation of [`get_optical_drive`].
///
/// This method gets the optional drive information from the operating system
/// for a drive with serial number `serial_number`. Returns `None` if an optical
/// drive cannot be found or an error if the command fails or its results cannot
/// be processed.
fn get_optical_drive_impl<F: Fn() -> Result<String>>(
    serial_number: &str,
    run_cmd: F,
) -> Result<Option<OsOpticalDrive>> {
    let json = run_cmd()?;

    let block_device_data = serde_json::from_str::<BlockDeviceData>(&json)
        .map_err(error::json_deserialize)?;

    for bd in block_device_data.block_devices {
        if !bd.is_optical_drive() {
            continue;
        }
        if bd.has_serial_number(serial_number) {
            return Ok(Some(bd.to_optical_drive()));
        }
    }

    Ok(None)
}

/// Gets the optical drive information for an optical drive with serial number
/// `serial_number`.
///
/// Returns `None` if an optical drive cannot be found with the provided serial
/// number. Returns an error if something goes wrong when querying the operating
/// system.
///
/// This is the Linux specific implementation.
pub fn get_optical_drive(serial_number: &str) -> Result<Option<OsOpticalDrive>> {
    get_optical_drive_impl(serial_number, run_lsblk_command)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod block_device {
        use super::*;

        #[test]
        fn test_has_serial_number() {
            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: Some(String::from("1234567890")),
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: Some(String::from("uuid-1234")),
            };
            assert!(device.has_serial_number(&String::from("1234567890")));
            assert!(!device.has_serial_number(&String::from("123456789")));

            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: None,
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: Some(String::from("uuid-1234")),
            };
            assert!(!device.has_serial_number(&String::from("1234567890")));
        }

        #[test]
        fn test_is_optical_drive() {
            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: Some(String::from("1234567890")),
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: Some(String::from("uuid-1234")),
            };
            assert!(device.is_optical_drive());

            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: Some(String::from("1234567890")),
                label: None,
                device_type: String::from("rom"),
                uuid: None,
            };
            assert!(device.is_optical_drive());

            let device = BlockDevice {
                name: String::from("/dev/sda"),
                serial_number: Some(String::from("1234567890")),
                label: Some(String::from("BOOT_DRIVE")),
                device_type: String::from("disk"),
                uuid: Some(String::from("uuid-1234")),
            };
            assert!(!device.is_optical_drive());

            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: Some(String::from("1234567890")),
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: None,
            };
            assert!(device.is_optical_drive());

            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: None,
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: Some(String::from("uuid-1234")),
            };
            assert!(!device.is_optical_drive());
        }

        #[test]
        fn test_to_optical_drive() {
            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: Some(String::from("1234567890")),
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: Some(String::from("uuid-1234")),
            };

            let optical_drive = device.to_optical_drive();
            assert_eq!(optical_drive.path, String::from("/dev/sr0"));
            assert_eq!(optical_drive.serial_number, String::from("1234567890"));
            match optical_drive.disc {
                DiscState::Inserted { label, uuid } => {
                    assert_eq!(label, String::from("Test Disc"));
                    assert_eq!(uuid, String::from("uuid-1234"));
                }
                _ => panic!("Expected disc state to be Inserted"),
            }

            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: Some(String::from("1234567890")),
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: None,
            };

            let optical_drive = device.to_optical_drive();
            assert_eq!(optical_drive.path, String::from("/dev/sr0"));
            assert_eq!(optical_drive.serial_number, String::from("1234567890"));
            match optical_drive.disc {
                DiscState::Inserted { label, uuid } => {
                    assert_eq!(label, String::from("Test Disc"));
                    assert_eq!(uuid, String::from(""));
                }
                _ => panic!("Expected disc state to be Inserted"),
            }

            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: Some(String::from("1234567890")),
                label: None,
                device_type: String::from("rom"),
                uuid: None,
            };

            let optical_drive = device.to_optical_drive();
            assert_eq!(optical_drive.path, String::from("/dev/sr0"));
            assert_eq!(optical_drive.serial_number, String::from("1234567890"));
            match optical_drive.disc {
                DiscState::None => (),
                _ => panic!("Expected disc state to be None"),
            }
        }

        #[test]
        #[should_panic]
        fn to_optical_drive_invalid_drive() {
            let device = BlockDevice {
                name: String::from("/dev/sr0"),
                serial_number: None,
                label: Some(String::from("Test Disc")),
                device_type: String::from("rom"),
                uuid: Some(String::from("uuid-1234")),
            };
            let _ = device.to_optical_drive();
        }
    }

    #[test]
    fn test_get_optical_drive() {
        let json = r###"
        {
            "blockdevices": [
                {
                    "name": "/dev/loop0",
                    "serial": null,
                    "type": "loop",
                    "label": null,
                    "uuid": null
                },
                {
                    "name": "/dev/sr0",
                    "serial": "SN0001",
                    "type": "rom",
                    "label": null,
                    "uuid": null
                },
                {
                    "name": "/dev/sr1",
                    "serial": "SN0002",
                    "type": "rom",
                    "label": "MOVIE",
                    "uuid": "4-8-15-16-23-42"
                },
                {
                    "name": "/dev/sda",
                    "serial": "SN0003",
                    "type": "disk",
                    "label": null,
                    "uuid": null
                }
            ]
        }"###;

        let cmd = || -> Result<String> { Ok(json.to_string()) };

        let optical_drive = match get_optical_drive_impl(&String::from("SN0001"), cmd) {
            Ok(result) => match result {
                Some(drive) => drive,
                None => panic!("Expected to find an optical drive with SN0001"),
            },
            _ => panic!("An unexpected error occurred"),
        };

        assert_eq!(optical_drive.path, String::from("/dev/sr0"));
        assert_eq!(optical_drive.serial_number, String::from("SN0001"));
        match optical_drive.disc {
            DiscState::None => (),
            _ => panic!("Expected disc state to be None"),
        }

        let optical_drive = match get_optical_drive_impl(&String::from("SN0002"), cmd) {
            Ok(result) => match result {
                Some(drive) => drive,
                None => panic!("Expected to find an optical drive with SN0002"),
            },
            _ => panic!("An unexpected error occurred"),
        };

        assert_eq!(optical_drive.path, String::from("/dev/sr1"));
        assert_eq!(optical_drive.serial_number, String::from("SN0002"));
        match optical_drive.disc {
            DiscState::Inserted { label, uuid } => {
                assert_eq!(label, String::from("MOVIE"));
                assert_eq!(uuid, String::from("4-8-15-16-23-42"));
            }
            _ => panic!("Expected disc state to be Inserted"),
        }

        match get_optical_drive_impl(&String::from("SN0003"), cmd) {
            Ok(result) => match result {
                Some(_) => panic!("Expected to not find an optical drive with SN0003"),
                None => (),
            },
            _ => panic!("An unexpected error occurred"),
        };

        match get_optical_drive_impl(&String::from("SN0004"), cmd) {
            Ok(result) => match result {
                Some(_) => panic!("Expected to not find an optical drive with SN0004"),
                None => (),
            },
            _ => panic!("An unexpected error occurred"),
        };
    }

    #[test]
    fn test_get_optical_drive_command_error() {
        let cmd = || -> Result<String> { 
            Err(Error::CommandReturnedErrorCode {
                command: String::from("test_command"),
                args: String::from("test_args"),
                code: Some(47), 
                stdout: String::from("stdout text"),
                stderr: String::from("stderr text") 
            })
        };
        match get_optical_drive_impl(&String::from("SN0001"), cmd) {
            Ok(_) => panic!("Expected an error when the command returns an error"),
            Err(err) => match err {
                Error::CommandReturnedErrorCode { command, args, code, stdout, stderr } => {
                    assert_eq!(command, "test_command");
                    assert_eq!(args, "test_args");
                    assert_eq!(code.unwrap(), 47);
                    assert_eq!(stdout, "stdout text");
                    assert_eq!(stderr, "stderr text");
                },
                _ => panic!("Expected the returned error to match the command error"),
            },
        }
    }

    #[test]
    fn test_get_optical_drive_invalid_json() {
        let json = r###"
        {
            "blockdevices": [
                {
                    "name": "/dev/loop0",
                    "serial": null,
                    "type": "loop",
                    "label": null,
                    "uuid": null
                }
        }"###;

        let cmd = || -> Result<String> { Ok(json.to_string()) };

        match get_optical_drive_impl(&String::from("SN0001"), cmd) {
            Ok(_) => panic!("Expected an error when the command returns an error"),
            Err(err) => match err {
                Error::Serialization { .. } => (),
                _ => panic!("Expected an error with invalid JSON"),
            },
        }
    }
}
