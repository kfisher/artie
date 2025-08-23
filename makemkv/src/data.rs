// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Defines the data types used to represent the information about the DVD or Blu-ray and its
//! contents extracted by MakeMKV.

/// Specifies the information attribute types that can be extracted by MakeMKV
/// by running the 'info' command.
///
/// These attributes can apply to either the disc, titles within the disc, or
/// streams (audio, subtitle, or video).
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

