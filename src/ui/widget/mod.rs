// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Custom UI widgets.

mod audio_track_field;
mod archive_form;
mod dropdown;
mod entry;
mod copy_form;
mod copy_page;
mod delete_form;
mod drive;
mod dual_icon_toggle_button;
mod icon_button;
mod icon_toggle_button;
mod title_form;
mod transcode_form;
mod transcode_list;
mod transcode_list_filter;
mod transcode_list_item;
mod transcode_page;
mod transcode_queue;
mod video_player;
mod window;

pub use audio_track_field::AudioTrackFieldWidget;
pub use archive_form::ArchiveFormWidget;
pub use copy_form::CopyFormWidget;
pub use copy_page::CopyPageWidget;
pub use delete_form::DeleteFormWidget;
pub use drive::DriveWidget;
pub use dropdown::DropDownWidget;
pub use dual_icon_toggle_button::DuelIconToggleButton;
pub use entry::EntryWidget;
pub use icon_button::IconButton;
pub use icon_toggle_button::IconToggleButton;
pub use title_form::TitleFormWidget;
pub use transcode_form::TranscodeFormWidget;
pub use transcode_list::TranscodeListWidget;
pub use transcode_list_filter::TranscodeListFilterWidget;
pub use transcode_list_item::TranscodeListItemWidget;
pub use transcode_page::TranscodePageWidget;
pub use transcode_queue::TranscodeQueueWidget;
pub use video_player::VideoPlayerWidget;
pub use window::Window;
