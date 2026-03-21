// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Conversion functions for converting models to and from database types.

use std::path::PathBuf;

use crate::models::{
    ContainerType,
    MediaType, 
    MediaLocation,
    OperationState, 
    SpecialFeature,
    SpecialFeatureType
};

/// Converts container type to its integral database value.
pub fn container_type_to_sql(container_type: &ContainerType) -> u8 {
    match container_type {
        ContainerType::MKV => 0,
        ContainerType::MP4 => 1,
    }
}

/// Converts media location to its database values.
/// 
/// The returned result will be a two value tuple where the first value is the numeric value
/// representing the area and the second is a string representing the path relative to the area's
/// root folder.
pub fn media_location_to_sql(media_location: &MediaLocation) -> (u8, String) {
    let (area, path) = match media_location {
        MediaLocation::Inbox(path) => (1, path.to_owned()),
        MediaLocation::Library(path) => (2, path.to_owned()),
        MediaLocation::Archive(path) => (3, path.to_owned()),
        MediaLocation::Deleted => (4, PathBuf::default()),
    };

    // TODO: Proper error handling?
    (area, path.to_str().unwrap().to_owned())
}

/// Converts media type to the integral value for use in the database.
pub fn media_type_to_sql(media_type: &MediaType) -> u8 {
    match media_type {
        MediaType::Movie => 0,
        MediaType::Show => 1,
    }
}

/// Converts operation state to its integral value for use in the database.
///
/// The result will be a two value tuple where the first value is the numberic value for the
/// operation state and the second value is the error message when the operation state is `Failed`.
/// For other operation states, it will be an empty string since the database column is not
/// nullable.
pub fn operation_state_to_sql(state: &OperationState) -> (u8, String) {
    match state {
        OperationState::Requested => (0, String::default()),
        OperationState::Running => (1, String::default()),
        OperationState::Completed => (2, String::default()),
        OperationState::Cancelled => (3, String::default()),
        OperationState::Failed { reason } => (4, reason.to_owned()),
    }
}

/// Converts special feature to its database values.
///
/// The returned result will be a two value tuple where the first value is the numeric value for 
/// the special feature type and the second value is the name of the special feature.
///
/// If the provided special feature is `None`, the result will be the default values for the
/// database indicating the its not a special feature.
pub fn special_feature_to_sql(special_feature: &Option<SpecialFeature>) -> (u8, String) {
    let Some(special_feature) = special_feature else {
        return (0, String::default())
    };

    match special_feature.kind {
        SpecialFeatureType::None => (0, String::default()),
        SpecialFeatureType::BehindTheScenes => (1, special_feature.name.clone()),
        SpecialFeatureType::DeletedScenes => (2, special_feature.name.clone()),
        SpecialFeatureType::Interviews => (3, special_feature.name.clone()),
        SpecialFeatureType::Scenes => (4, special_feature.name.clone()),
        SpecialFeatureType::Samples => (5, special_feature.name.clone()),
        SpecialFeatureType::Shorts => (6, special_feature.name.clone()),
        SpecialFeatureType::Featurettes => (7, special_feature.name.clone()),
        SpecialFeatureType::Clips => (8, special_feature.name.clone()),
        SpecialFeatureType::Extras => (9, special_feature.name.clone()),
        SpecialFeatureType::Trailers => (10, special_feature.name.clone()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::models::{
        ContainerType,
        MediaLocation,
        MediaType,
        OperationState,
        SpecialFeature,
        SpecialFeatureType,
    };

    #[test]
    fn test_container_type_to_sql_mkv() {
        assert_eq!(container_type_to_sql(&ContainerType::MKV), 0);
    }

    #[test]
    fn test_container_type_to_sql_mp4() {
        assert_eq!(container_type_to_sql(&ContainerType::MP4), 1);
    }

    #[test]
    fn test_media_type_to_sql_movie() {
        assert_eq!(media_type_to_sql(&MediaType::Movie), 0);
    }

    #[test]
    fn test_media_type_to_sql_show() {
        assert_eq!(media_type_to_sql(&MediaType::Show), 1);
    }

    #[test]
    fn test_media_location_to_sql_inbox() {
        let (area, path) = media_location_to_sql(
            &MediaLocation::Inbox(PathBuf::from("movies/foo.mkv"))
        );
        assert_eq!(area, 1);
        assert_eq!(path, "movies/foo.mkv");
    }

    #[test]
    fn test_media_location_to_sql_library() {
        let (area, path) = media_location_to_sql(
            &MediaLocation::Library(PathBuf::from("shows/bar.mkv"))
        );
        assert_eq!(area, 2);
        assert_eq!(path, "shows/bar.mkv");
    }

    #[test]
    fn test_media_location_to_sql_archive() {
        let (area, path) = media_location_to_sql(
            &MediaLocation::Archive(PathBuf::from("archive/baz.mkv"))
        );
        assert_eq!(area, 3);
        assert_eq!(path, "archive/baz.mkv");
    }

    #[test]
    fn test_media_location_to_sql_deleted() {
        let (area, path) = media_location_to_sql(&MediaLocation::Deleted);
        assert_eq!(area, 4);
        assert_eq!(path, "");
    }

    #[test]
    fn test_operation_state_to_sql_requested() {
        let (state, error) = operation_state_to_sql(&OperationState::Requested);
        assert_eq!(state, 0);
        assert!(error.is_empty());
    }

    #[test]
    fn test_operation_state_to_sql_running() {
        let (state, error) = operation_state_to_sql(&OperationState::Running);
        assert_eq!(state, 1);
        assert!(error.is_empty());
    }

    #[test]
    fn test_operation_state_to_sql_completed() {
        let (state, error) = operation_state_to_sql(&OperationState::Completed);
        assert_eq!(state, 2);
        assert!(error.is_empty());
    }

    #[test]
    fn test_operation_state_to_sql_cancelled() {
        let (state, error) = operation_state_to_sql(&OperationState::Cancelled);
        assert_eq!(state, 3);
        assert!(error.is_empty());
    }

    #[test]
    fn test_operation_state_to_sql_failed() {
        let reason = "disk full".to_owned();
        let (state, error) = operation_state_to_sql(
            &OperationState::Failed { reason: reason.clone() }
        );
        assert_eq!(state, 4);
        assert_eq!(error, reason);
    }

    #[test]
    fn test_special_feature_to_sql_none_option() {
        let (kind, name) = special_feature_to_sql(&None);
        assert_eq!(kind, 0);
        assert!(name.is_empty());
    }

    #[test]
    fn test_special_feature_to_sql_none_kind() {
        let sf = Some(SpecialFeature { kind: SpecialFeatureType::None, name: String::new() });
        let (kind, name) = special_feature_to_sql(&sf);
        assert_eq!(kind, 0);
        assert!(name.is_empty());
    }

    #[test]
    fn test_special_feature_to_sql_types() {
        let cases = [
            (SpecialFeatureType::BehindTheScenes, 1u8),
            (SpecialFeatureType::DeletedScenes, 2),
            (SpecialFeatureType::Interviews, 3),
            (SpecialFeatureType::Scenes, 4),
            (SpecialFeatureType::Samples, 5),
            (SpecialFeatureType::Shorts, 6),
            (SpecialFeatureType::Featurettes, 7),
            (SpecialFeatureType::Clips, 8),
            (SpecialFeatureType::Extras, 9),
            (SpecialFeatureType::Trailers, 10),
        ];

        for (feature_type, expected_kind) in cases {
            let sf = Some(SpecialFeature { kind: feature_type, name: "Test Feature".to_owned() });
            let (kind, name) = special_feature_to_sql(&sf);
            assert_eq!(kind, expected_kind);
            assert_eq!(name, "Test Feature");
        }
    }
}
