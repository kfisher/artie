// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

/// Represents the messages that are outputted by MakeMKV when running its
/// various commands.
enum Message {
    /// CINFO messages contain information about a disc inserted into a drive.
    /// Each message is a key/value pair representing a single attribute of a
    /// disc.
    Cinfo {
        /// The attribute identifier.
        id: i32,

        /// Unique message code if the value is a constant string; zero
        /// otherwise. This value does not have any use outside of MakeMKV.
        code: i32,

        /// The attribute value.
        value: String,
    },

    /// DRV messages contain information about an optical drive. This
    /// information is similar to, but may vary slightly from the data for
    /// optical drives from the optical drive package.
    Drv {
        /// The index of the drive assigned by MakeMKV.
        index: i32,

        /// The purpose of this value is not currently known. It is included as
        /// a placeholder should its purpose ever become known.
        unkown: i32,

        /// Flags describing certain characteristics about the type of disc and
        /// its content inserted into the drive.
        media_flags: i32,

        /// The name of the drive which is derived from the drive's manufacturer
        /// and serial number.
        drive_name: String,

        /// The name of the disc inserted into the drive or an empty string if
        /// there is not a disc in the drive.
        disc_name: String,

        /// The device path of the drive.
        device_path: String,
    },

    /// MSG messages are general information messages.
    Msg {
        /// Unique message code. This does not appear to have any real use
        /// outside of MakeMKV.
        code: i32,

        /// Message flags.
        flags: i32,

        /// The number of message arguments.
        count: i32,

        /// The complete formatted message. This is essentially `format` with
        /// its placeholders replaced by the arguments in `args`.
        message: String,

        /// The message format string.
        format: String,

        /// The message arguments.
        args: Vec<String>,
    },

    /// PRGC messages contain the name of the current suboperation.
    Prgc {
        /// Unique message code. This does not appear to have any real use
        /// outside of MakeMKV.
        code: i32,

        /// The suboperation index (or unique id).
        id: i32,

        /// The suboperation title.
        name: String,
    },

    /// PRGT messages contain the name of the current operation.
    Prgt {
        /// Unique message code. This does not appear to have any real use
        /// outside of MakeMKV.
        code: i32,

        /// The operation index (or unique id).
        id: i32,

        /// The operation title.
        name: String,
    },

    /// PRGV messages contain the progress of the current operation and
    /// suboperation.
    Prgv {
        /// The progress of the current suboperation.
        suboperation: i32,

        /// The progress of the current operation.
        operation: i32,

        /// The value at which current or progress is considered 100% complete.
        /// `% complete = [sub]operation / max`
        max: i32,
    },

    /// SINFO messages contain information about an audio, subtitle, or video
    /// stream within a title on the disc. Each message is a key/value pair
    /// representing a single attribute of a stream.
    Sinfo {
        /// The index of the title the stream belongs to.
        title_index: i32,

        /// The index of the stream within the disc.
        stream_index: i32,

        /// The attribute identifier.
        id: i32,

        /// Unique message code if the value is a constant string; zero
        /// otherwise. This value does not have any use outside of MakeMKV.
        code: i32,

        /// The attribute value.
        value: String,
    },

    /// TCOUNT messages contain the total number of titles. This may or may not
    /// match the number of titles MKV videos are created for since some titles
    /// may be omitted because they do not meet the min or max length.
    Tcount {
        /// The total number of titles on the disc.
        count: i32,
    },

    /// TINFO messages contain information about a title on the disc. Each
    /// message is a key/value pair representing a single attribute of a title.
    Tinfo {
        /// The index of the title within the disc.
        title_index: i32,

        /// The attribute identifier.
        id: i32,

        /// Unique message code if the value is a constant string; zero
        /// otherwise. This value does not have any use outside of MakeMKV.
        code: i32,

        /// The attribute value.
        value: String,
    },
}
