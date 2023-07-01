//! OXD File extension structure

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    asset::{GetAssets, ReplaceAsset},
    storage::{StorageId, StorageIdWithoutSerde},
    OXD_VERSION,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct OxdXml<A: StorageIdWithoutSerde> {
    pub version: String,
    pub screens: Vec<Screen<A>>,
}

impl<A: StorageId> OxdXml<A> {
    pub fn new() -> OxdXml<A> {
        OxdXml {
            version: String::from(OXD_VERSION),
            screens: vec![]
        }
    }
}

impl<A: StorageId, B: StorageId> ReplaceAsset<B> for OxdXml<A> {
    type From = A;

    type Output = OxdXml<B>;

    fn replace_asset<'a>(self, assets: &'a mut std::collections::HashMap<A, B>) -> Self::Output {
        let mut new_screens: Vec<Screen<B>> = vec![];

        for old_screen in self.screens {
            new_screens.push(old_screen.replace_asset(assets));
        }
        OxdXml {
            version: self.version,
            screens: new_screens 
        }
    }
}

impl<A: StorageId> GetAssets<A> for OxdXml<A> {
    fn get_assets(&self) -> Vec<A> {
        return vec![];
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Screen<A: StorageIdWithoutSerde> {
    _phantom: PhantomData<A>,
}

impl<A: StorageId, B: StorageId> ReplaceAsset<B> for Screen<A> {
    type From = A;

    type Output = Screen<B>;

    fn replace_asset<'a>(self, assets: &'a mut std::collections::HashMap<A, B>) -> Self::Output {
        Screen {
            _phantom: PhantomData
        }
    }
}


pub struct ScreenSize {
    model: String,
    device_name: String,
    /// Width in mm
    width: f32,
    /// Height in mm
    height: f32,
    /// Resolution wxh
    resolution: (u32, u32),
    /// The shape of the outside of screen
    outer_path: String,
    /// The shape of the inside of screen
    inner_path: String,
}
