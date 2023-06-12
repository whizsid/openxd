//! Asset file related functions

use std::collections::HashMap;

use crate::storage::StorageId;

/// Allowed asset types
pub enum AssetType {
    JPG,
    JPEG,
    GIF
}

pub fn detect_asset_type_by_ext(ext: &str) -> Option<AssetType> {
    match ext {
        "jpg" => Some(AssetType::JPG),
        "jpeg" => Some(AssetType::JPEG),
        "gif" => Some(AssetType::GIF),
        _ => None
    }
}

pub trait ReplaceAsset<TO: StorageId > {
    type From: StorageId;

    type Output;
    /// Replace asset URLs/ids
    fn replace_asset<'a>(self, assets: &'a mut HashMap<Self::From, TO>)-> Self::Output;
}

pub trait GetAssets<S: StorageId> {
    fn get_assets(&self) -> Vec<S>;
}
