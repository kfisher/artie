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

// TODO: TESTING
