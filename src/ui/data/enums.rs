// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject enumerations.

use gtk::glib;

/// GObject representation of [`crate::models::MediaType`].
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(u8)]
#[enum_type(name = "ArtieMediaType")]
pub enum MediaType {
    #[default]
    Movie = 0,
    Show = 1,
}

impl MediaType {
    pub fn to_model(&self) -> crate::models::MediaType {
        match self {
            MediaType::Movie => crate::models::MediaType::Movie,
            MediaType::Show => crate::models::MediaType::Show,
        }
    }
}

impl From<crate::models::MediaType> for MediaType {
    fn from(value: crate::models::MediaType) -> Self {
        match value {
            crate::models::MediaType::Movie => MediaType::Movie,
            crate::models::MediaType::Show => MediaType::Show,
        }
    }
}

/// GObject representation of the optical drive's state.
///
/// This state combines the [`crate::drive::DiscState`] and [`crate::drive::OpticalDriveState`]
/// rust types into a single enumeration that can be used as a GObject property.
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(u8)]
#[enum_type(name = "ArtieOpticalDriveState")]
pub enum OpticalDriveState {
    #[default]
    Disconnected = 0,
    Empty = 1,
    Idle = 2,
    Copying = 3,
    Success = 4,
    Failed = 5,
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(u8)]
#[enum_type(name = "ArtieSpecialFeatureType")]
pub enum SpecialFeatureType {
    #[default]
    None = 0,
    BehindTheScenes = 1,
    DeletedScenes = 2,
    Interviews = 3,
    Scenes = 4,
    Samples = 5,
    Shorts = 6,
    Featurettes = 7,
    Clips = 8,
    Extras = 9,
    Trailers = 10,
}

impl SpecialFeatureType {
    pub fn to_model(&self) -> crate::models::SpecialFeatureType {
        match self {
            SpecialFeatureType::None  => crate::models::SpecialFeatureType::None,
            SpecialFeatureType::BehindTheScenes  => crate::models::SpecialFeatureType::BehindTheScenes,
            SpecialFeatureType::DeletedScenes  => crate::models::SpecialFeatureType::DeletedScenes,
            SpecialFeatureType::Interviews  => crate::models::SpecialFeatureType::Interviews,
            SpecialFeatureType::Scenes  => crate::models::SpecialFeatureType::Scenes,
            SpecialFeatureType::Samples  => crate::models::SpecialFeatureType::Samples,
            SpecialFeatureType::Shorts  => crate::models::SpecialFeatureType::Shorts,
            SpecialFeatureType::Featurettes  => crate::models::SpecialFeatureType::Featurettes,
            SpecialFeatureType::Clips  => crate::models::SpecialFeatureType::Clips,
            SpecialFeatureType::Extras  => crate::models::SpecialFeatureType::Extras,
            SpecialFeatureType::Trailers  => crate::models::SpecialFeatureType::Trailers,
        }
    }

    pub fn to_special_feature(&self, name: &str) -> Option<crate::models::SpecialFeature> {
        let special_feature_type = self.to_model();
        if special_feature_type.is_none() {
            None
        } else {
            let special_feature = crate::models::SpecialFeature {
                kind: special_feature_type,
                name: name.to_owned(),
            };
            Some(special_feature)
        }
    }
}

impl From<crate::models::SpecialFeatureType> for SpecialFeatureType {
    fn from(value: crate::models::SpecialFeatureType) -> Self {
        match value {
            crate::models::SpecialFeatureType::None => SpecialFeatureType::None,
            crate::models::SpecialFeatureType::BehindTheScenes => SpecialFeatureType::BehindTheScenes,
            crate::models::SpecialFeatureType::DeletedScenes => SpecialFeatureType::DeletedScenes,
            crate::models::SpecialFeatureType::Interviews => SpecialFeatureType::Interviews,
            crate::models::SpecialFeatureType::Scenes => SpecialFeatureType::Scenes,
            crate::models::SpecialFeatureType::Samples => SpecialFeatureType::Samples,
            crate::models::SpecialFeatureType::Shorts => SpecialFeatureType::Shorts,
            crate::models::SpecialFeatureType::Featurettes => SpecialFeatureType::Featurettes,
            crate::models::SpecialFeatureType::Clips => SpecialFeatureType::Clips,
            crate::models::SpecialFeatureType::Extras => SpecialFeatureType::Extras,
            crate::models::SpecialFeatureType::Trailers => SpecialFeatureType::Trailers,
        }
    }
}

