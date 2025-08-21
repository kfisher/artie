// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Handles parsing messages from MakeMKV.
//!
//! When running MakeMKV from the command-line, MakeMKV will output messages to
//! the console where each line is a separate message. This module provides the
//! utilities for parsing those messages.
//!
//! The message structure was constructed using the documentation provided in
//! <https://gist.github.com/csandman/ad221b9014cf88c29ccfa604d8507790> as well
//! as data found in the source code (header files) provided in the open source
//! parts of MakeMKV (version 1.7.17).

// TODO: Revisit this once the MakeMKV crate has been completed as well as the
//       crate(s) that will use it. There are a lot of warnings and I don't want
//       to remove the code until I'm sure it won't be needed.
#![allow(dead_code)]

/// Specifies the information attribute types that can be extracted by MakeMKV
/// by running the 'info' command.
///
/// These attributes can apply to either the disc, titles within the disc, or
/// streams (audio, subtitle, or video). The following MakeMKV messages contain
/// these attributes:
///
/// - [`Message::Cinfo`] Disc Info
/// - [`Message::Tinfo`] Title Info
/// - [`Message::Sinfo`] Stream Info
#[derive(Debug, PartialEq)]
pub enum Attribute {
    Unknown,
    Type,
    Name,
    LangCode,
    LangName,
    CodecId,
    CodecShort,
    CodecLong,
    ChapterCount,
    Duration,
    DiskSize,
    DiskSizeBytes,
    StreamTypeExtension,
    Bitrate,
    AudioChannelsCount,
    AngleInfo,
    SourceFileName,
    AudioSampleRate,
    AudioSampleSize,
    VideoSize,
    VideoAspectRatio,
    VideoFrameRate,
    StreamFlags,
    DateTime,
    OriginalTitleId,
    SegmentsCount,
    SegmentsMap,
    OutputFileName,
    MetadataLanguageCode,
    MetadataLanguageName,
    TreeInfo,
    PanelTitle,
    VolumeName,
    OrderWeight,
    OutputFormat,
    OutputFormatDescription,
    SeamlessInfo,
    PanelText,
    MkvFlags,
    MkvFlagsText,
    AudioChannelLayoutName,
    OutputCodecShort,
    OutputConversionType,
    OutputAudioSampleRate,
    OutputAudioSampleSize,
    OutputAudioChannelsCount,
    OutputAudioChannelLayoutName,
    OutputAudioChannelLayout,
    OutputAudioMixDescription,
    Comment,
    OffsetSequenceId,
}

impl Attribute {
    /// Converts the numberic value used to represent these attributes in
    /// MakeMKV to its corresponding `Attribute` value.
    ///
    /// Returns [`Error::AttributeConversion`] if the value could not be
    /// converted.
    pub fn from_i32(n: i32) -> Result<Self, Error> {
        match n {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Type),
            2 => Ok(Self::Name),
            3 => Ok(Self::LangCode),
            4 => Ok(Self::LangName),
            5 => Ok(Self::CodecId),
            6 => Ok(Self::CodecShort),
            7 => Ok(Self::CodecLong),
            8 => Ok(Self::ChapterCount),
            9 => Ok(Self::Duration),
            10 => Ok(Self::DiskSize),
            11 => Ok(Self::DiskSizeBytes),
            12 => Ok(Self::StreamTypeExtension),
            13 => Ok(Self::Bitrate),
            14 => Ok(Self::AudioChannelsCount),
            15 => Ok(Self::AngleInfo),
            16 => Ok(Self::SourceFileName),
            17 => Ok(Self::AudioSampleRate),
            18 => Ok(Self::AudioSampleSize),
            19 => Ok(Self::VideoSize),
            20 => Ok(Self::VideoAspectRatio),
            21 => Ok(Self::VideoFrameRate),
            22 => Ok(Self::StreamFlags),
            23 => Ok(Self::DateTime),
            24 => Ok(Self::OriginalTitleId),
            25 => Ok(Self::SegmentsCount),
            26 => Ok(Self::SegmentsMap),
            27 => Ok(Self::OutputFileName),
            28 => Ok(Self::MetadataLanguageCode),
            29 => Ok(Self::MetadataLanguageName),
            30 => Ok(Self::TreeInfo),
            31 => Ok(Self::PanelTitle),
            32 => Ok(Self::VolumeName),
            33 => Ok(Self::OrderWeight),
            34 => Ok(Self::OutputFormat),
            35 => Ok(Self::OutputFormatDescription),
            36 => Ok(Self::SeamlessInfo),
            37 => Ok(Self::PanelText),
            38 => Ok(Self::MkvFlags),
            39 => Ok(Self::MkvFlagsText),
            40 => Ok(Self::AudioChannelLayoutName),
            41 => Ok(Self::OutputCodecShort),
            42 => Ok(Self::OutputConversionType),
            43 => Ok(Self::OutputAudioSampleRate),
            44 => Ok(Self::OutputAudioSampleSize),
            45 => Ok(Self::OutputAudioChannelsCount),
            46 => Ok(Self::OutputAudioChannelLayoutName),
            47 => Ok(Self::OutputAudioChannelLayout),
            48 => Ok(Self::OutputAudioMixDescription),
            49 => Ok(Self::Comment),
            50 => Ok(Self::OffsetSequenceId),
            _ => Err(Error::AttributeConversion),
        }
    }
}

/// Errors that can occur while parsing MakeMKV messages.
pub enum Error {
    /// The message could not be split into its key and data components.
    InvalidMessageFormat,

    /// The data component of the message is missing data or was not able to be
    /// parsed into its data elements.
    InvalidDataFormat,

    /// Support for the message type has not be added yet.
    UnknownMessageType(String),

    /// Failed to convert a data element to an integer.
    IntConversion,

    /// Failed to convert a data element to an attribute.
    AttributeConversion,
}

/// Represents the messages that are outputted by MakeMKV when running its
/// various commands.
pub enum Message {
    /// CINFO messages contain information about a disc inserted into a drive.
    /// Each message is a key/value pair representing a single attribute of a
    /// disc.
    Cinfo {
        /// The attribute identifier.
        id: Attribute,

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

        /// The current state of the drive.
        ///
        /// - EMPTY_CLOSED = 0
        /// - EMPTY_OPEN = 1
        /// - INSERTED = 2
        /// - LOADING = 3
        /// - NO_DRIVE = 256
        /// - UNMOUNTING = 257
        state: i32,

        /// The purpose of this value is not currently known. It is included as
        /// a placeholder should its purpose ever become known.
        unknown: i32,

        /// Flags describing certain characteristics about the type of disc and
        /// its content inserted into the drive.
        ///
        /// - DVD_FILES_PRESENT = 1,
        ///   + Digital Video Disc (DVD)
        ///   + <https://en.wikipedia.org/wiki/DVD>
        /// - HDVD_FILES_PRESENT = 2,
        ///   + High-Definition Video Disc (HDVD)
        ///   + <https://en.wikipedia.org/wiki/High-Definition_Versatile_Disc>
        /// - BLURAY_FILES_PRESENT = 4,
        ///   + Blu-ray Disc
        ///   + <https://en.wikipedia.org/wiki/Blu-ray>
        /// - AACS_FILES_PRESENT = 8,
        ///   + Advanced Access Content System (AACS)
        ///   + <https://en.wikipedia.org/wiki/Advanced_Access_Content_System>
        /// - BDSVM_FILES_PRESENT = 16
        ///   + Blu-ray Disc Secure Video Path (BDSVP)
        ///   + <https://en.wikipedia.org/wiki/BD%2B>
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
        id: Attribute,

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
        id: Attribute,

        /// Unique message code if the value is a constant string; zero
        /// otherwise. This value does not have any use outside of MakeMKV.
        code: i32,

        /// The attribute value.
        value: String,
    },
}

/// Parses a message from MakeMKV.
///
/// Each message is expected to follow the pattern `KEY:DATA`. Key identifies
/// the type of message and data is a comma separated list of data which varies
/// based on the message type. See [`Message`] for the set of known message
/// types.
pub fn parse_message(raw_message: &str) -> Result<Message, Error> {
    let Some((key, data)) = raw_message.split_once(':') else {
        return Err(Error::InvalidMessageFormat);
    };
    match key {
        "CINFO" => parse_cinfo_message(data),
        "DRV" => parse_drv_message(data),
        "MSG" => parse_msg_message(data),
        "PRGC" => parse_prgc_message(data),
        "PRGT" => parse_prgt_message(data),
        "PRGV" => parse_prgv_message(data),
        "SINFO" => parse_sinfo_message(data),
        "TCOUNT" => parse_tcount_message(data),
        "TINFO" => parse_tinfo_message(data),
        _ => Err(Error::UnknownMessageType(String::from(key))),
    }
}

/// Parses the data component of a CINFO MakeMKV message.
///
/// Returns a [`Message::Cinfo`] message or an error if the provided data could
/// not be parsed.
fn parse_cinfo_message(data: &str) -> Result<Message, Error> {
    // Message: "CINFO:<id>,<code>,<value>"
    //          | Key | Data              |
    let mut parts = data.split(',');

    let Some(id) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(id) = id.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let id = Attribute::from_i32(id)?;

    let Some(code) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(code) = code.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(value) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let value = String::from(value.trim_matches('"'));

    Ok(Message::Cinfo { id, code, value })
}

/// Parses the data component of a DRV MakeMKV message.
///
/// Returns a [`Message::Drv`] message or an error if the provided data could
/// not be parsed.
fn parse_drv_message(data: &str) -> Result<Message, Error> {
    // Message: "DRV:<index>,<state>,<unknown>,<media_flags>,<drive_name>,<disc_name>,<device_path>"
    let mut parts = data.split(',');

    let Some(index) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(index) = index.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(state) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(state) = state.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(unknown) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(unknown) = unknown.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(media_flags) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(media_flags) = media_flags.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(drive_name) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let drive_name = String::from(drive_name.trim_matches('"'));

    let Some(disc_name) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let disc_name = String::from(disc_name.trim_matches('"'));

    let Some(device_path) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let device_path = String::from(device_path.trim_matches('"'));

    Ok(Message::Drv {
        index,
        state,
        unknown,
        media_flags,
        drive_name,
        disc_name,
        device_path,
    })
}

/// Parses the data component of a MSG MakeMKV message.
///
/// Returns a [`Message::Msg`] message or an error if the provided data could
/// not be parsed.
fn parse_msg_message(data: &str) -> Result<Message, Error> {
    // Message: "MSG:<code>,<flags>,<count>,<message>,<format>,<args..>"
    //          | Key | Data                                           |
    let mut parts = data.split(',');

    let Some(code) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(code) = code.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(flags) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(flags) = flags.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(count) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(count) = count.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(message) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let message = String::from(message.trim_matches('"'));

    let Some(format) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let format = String::from(format.trim_matches('"'));

    let args: Vec<String> = parts.map(String::from).collect();

    Ok(Message::Msg {
        code,
        flags,
        count,
        message,
        format,
        args,
    })
}

/// Parses the data component of a PRGC MakeMKV message.
///
/// Returns a [`Message::Prgc`] message or an error if the provided data could
/// not be parsed.
fn parse_prgc_message(data: &str) -> Result<Message, Error> {
    // Message: "PRGC:<code>,<id>,<name>"
    //          | Key | Data            |
    let mut parts = data.split(',');

    let Some(code) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(code) = code.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(id) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(id) = id.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(name) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let name = String::from(name.trim_matches('"'));

    Ok(Message::Prgc { code, id, name })
}

/// Parses the data component of a PRGT MakeMKV message.
///
/// Returns a [`Message::Prgt`] message or an error if the provided data could
/// not be parsed.
fn parse_prgt_message(data: &str) -> Result<Message, Error> {
    // Message: "PRGT:<code>,<id>,<name>"
    //          | Key | Data            |
    let mut parts = data.split(',');

    let Some(code) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(code) = code.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(id) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(id) = id.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(name) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let name = String::from(name.trim_matches('"'));

    Ok(Message::Prgt { code, id, name })
}

/// Parses the data component of a PRGV MakeMKV message.
///
/// Returns a [`Message::Prgv`] message or an error if the provided data could
/// not be parsed.
fn parse_prgv_message(data: &str) -> Result<Message, Error> {
    // Message: "PRGV:<suboperation>,<operation>,<max>"
    //          | Key | Data                          |
    let mut parts = data.split(',');

    let Some(suboperation) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(suboperation) = suboperation.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(operation) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(operation) = operation.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(max) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(max) = max.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    Ok(Message::Prgv {
        suboperation,
        operation,
        max,
    })
}

/// Parses the data component of a SINFO MakeMKV message.
///
/// Returns a [`Message::Sinfo`] message or an error if the provided data could
/// not be parsed.
fn parse_sinfo_message(data: &str) -> Result<Message, Error> {
    // Message: "SINFO:<title_index>,<stream_index>,<id>,<code>,<value>"
    //          | Key | Data                                           |
    let mut parts = data.split(',');

    let Some(title_index) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(title_index) = title_index.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(stream_index) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(stream_index) = stream_index.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(id) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(id) = id.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let id = Attribute::from_i32(id)?;

    let Some(code) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(code) = code.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(value) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let value = String::from(value.trim_matches('"'));

    Ok(Message::Sinfo {
        title_index,
        stream_index,
        id,
        code,
        value,
    })
}

/// Parses the data component of a TCOUNT MakeMKV message.
///
/// Returns a [`Message::Tcount`] message or an error if the provided data could
/// not be parsed.
fn parse_tcount_message(data: &str) -> Result<Message, Error> {
    // Message: "TCOUNT:<count>"
    //          | Key | Data   |
    match data.parse::<i32>() {
        Ok(count) => Ok(Message::Tcount { count }),
        Err(_) => Err(Error::IntConversion),
    }
}

/// Parses the data component of a TINFO MakeMKV message.
///
/// Returns a [`Message::Tinfo`] message or an error if the provided data could
/// not be parsed.
fn parse_tinfo_message(data: &str) -> Result<Message, Error> {
    // Message: "TINFO:<title_index>,<id>,<code>,<value>"
    //          | Key | Data                            |
    let mut parts = data.split(',');

    let Some(title_index) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(title_index) = title_index.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(id) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(id) = id.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let id = Attribute::from_i32(id)?;

    let Some(code) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let Ok(code) = code.parse::<i32>() else {
        return Err(Error::IntConversion);
    };

    let Some(value) = parts.next() else {
        return Err(Error::InvalidDataFormat);
    };

    let value = String::from(value.trim_matches('"'));

    Ok(Message::Tinfo {
        title_index,
        id,
        code,
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cinfo_message() {
        let msg = "CINFO:2,0,\"The Movie\"";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Cinfo { id, code, value } = msg else {
            panic!("Expected a Cinfo message to be returned")
        };

        assert_eq!(id, Attribute::Name);
        assert_eq!(code, 0);
        assert_eq!(value, "The Movie");
    }

    #[test]
    fn parse_drv_message() {
        let msg = "DRV:2,1,999,12,\"4815162342\",\"MOVIE\",\"/dev/sr1\"";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Drv {
            index,
            state,
            unknown,
            media_flags,
            drive_name,
            disc_name,
            device_path,
        } = msg
        else {
            panic!("Expected a Drv message to be returned")
        };

        assert_eq!(index, 2);
        assert_eq!(state, 1);
        assert_eq!(unknown, 999);
        assert_eq!(media_flags, 12);
        assert_eq!(drive_name, "4815162342");
        assert_eq!(disc_name, "MOVIE");
        assert_eq!(device_path, "/dev/sr1");
    }

    #[test]
    fn parse_msg_message() {
        let msg =
            "MSG:3007,0,0,\"Using direct disc access mode\",\"Using direct disc access mode\"";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Msg {
            code,
            flags,
            count,
            message,
            format,
            args,
        } = msg
        else {
            panic!("Expected a Msg message to be returned")
        };

        assert_eq!(code, 3007);
        assert_eq!(flags, 0);
        assert_eq!(count, 0);
        assert_eq!(message, "Using direct disc access mode");
        assert_eq!(format, "Using direct disc access mode");
        assert_eq!(args.len(), 0);
    }

    #[test]
    fn parse_prgc_message() {
        let msg = "PRGC:3400,7,\"Processing AV clips\"";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Prgc { code, id, name } = msg else {
            panic!("Expected a Prgc message to be returned")
        };

        assert_eq!(code, 3400);
        assert_eq!(id, 7);
        assert_eq!(name, "Processing AV clips");
    }

    #[test]
    fn parse_prgt_message() {
        let msg = "PRGT:3400,7,\"Processing AV clips\"";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Prgt { code, id, name } = msg else {
            panic!("Expected a Prgt message to be returned")
        };

        assert_eq!(code, 3400);
        assert_eq!(id, 7);
        assert_eq!(name, "Processing AV clips");
    }

    #[test]
    fn parse_prgv_message() {
        let msg = "PRGV:30929,21318,65536";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Prgv {
            suboperation,
            operation,
            max,
        } = msg
        else {
            panic!("Expected a Prgv message to be returned")
        };

        assert_eq!(suboperation, 30929);
        assert_eq!(operation, 21318);
        assert_eq!(max, 65536);
    }

    #[test]
    fn parse_sinfo_message() {
        let msg = "SINFO:5,1,7,0,\"Dolby Digital\"";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Sinfo {
            title_index,
            stream_index,
            id,
            code,
            value,
        } = msg
        else {
            panic!("Expected a Sinfo message to be returned")
        };

        assert_eq!(title_index, 5);
        assert_eq!(stream_index, 1);
        assert_eq!(id, Attribute::CodecLong);
        assert_eq!(code, 0);
        assert_eq!(value, "Dolby Digital");
    }

    #[test]
    fn parse_tcount_message() {
        let msg = "TCOUNT:53";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Tcount { count } = msg else {
            panic!("Expected a Tcount message to be returned")
        };

        assert_eq!(count, 53);
    }

    #[test]
    fn parse_tinfo_message() {
        let msg = "TINFO:3,27,0,\"MOVIE_t00.mkv\"";

        let Ok(msg) = parse_message(msg) else {
            panic!("Expected a valid message to be returned")
        };

        let Message::Tinfo {
            title_index,
            id,
            code,
            value,
        } = msg
        else {
            panic!("Expected a Tinfo message to be returned")
        };

        assert_eq!(title_index, 3);
        assert_eq!(id, Attribute::OutputFileName);
        assert_eq!(code, 0);
        assert_eq!(value, "MOVIE_t00.mkv");
    }

    #[test]
    fn parse_invalid_message() {
        let invalid_data = [
            "UNKNOWN:0,0,0",
            "INVALID",
            "CINFO:INVALID,0,\"The Movie\"",
            "CINFO:5000,0,\"The Movie\"",
            "CINFO:-500,0,\"The Movie\"",
            "CINFO:2,0",
            "DRV:INVALID,1,999,12,\"4815162342\",\"MOVIE\",\"/dev/sr1\"",
            "DRV:2,INVALID,999,12,\"4815162342\",\"MOVIE\",\"/dev/sr1\"",
            "DRV:2,1,999,INVALID2,\"4815162342\",\"MOVIE\",\"/dev/sr1\"",
            "DRV:2,1,999,12,\"MOVIE\",\"/dev/sr1\"",
            "MSG:INVALID,0,0,\"Using direct disc access mode\",\"Using direct disc access mode\"",
            "MSG:3007",
            "PRGC:INVALID,7,\"Processing AV clips\"",
            "PRGC:3400,INVALID,\"Processing AV clips\"",
            "PRGC:3400,7",
            "PRGT:INVALID,9,\"Opening Blu-ray disc\"",
            "PRGT:3404,INVALID,\"Opening Blu-ray disc\"",
            "PRGT:3404,9",
            "PRGV:INVALID,21318,65536",
            "PRGV:30929,INVALID,65536",
            "PRGV:30929,21318,INVALID",
            "PRGV:30929,21318",
            "SINFO:INVALID,1,7,0,\"Dolby Digital\"",
            "SINFO:5,INVALID,7,0,\"Dolby Digital\"",
            "SINFO:5,1,INVALID,0,\"Dolby Digital\"",
            "SINFO:5,1,3000,0,\"Dolby Digital\"",
            "SINFO:5,1,-300,0,\"Dolby Digital\"",
            "SINFO:5",
            "TCOUNT:INVALID",
            "TCOUNT:",
            "TINFO:INVALID,27,0,\"MOVIE_t00.mkv\"",
            "TINFO:3,INVALID,0,\"MOVIE_t00.mkv\"",
            "TINFO:3,2000,0,\"MOVIE_t00.mkv\"",
            "TINFO:3,-200,0,\"MOVIE_t00.mkv\"",
            "TINFO:3",
        ];

        for item in invalid_data.iter() {
            let Err(_) = parse_message(item) else {
                panic!("Expected an error when parsing '{}'", item);
            };
        }
    }
}
