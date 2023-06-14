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
    _phantom: PhantomData<A>,
}

impl<A: StorageId> OxdXml<A> {
    pub fn new() -> OxdXml<A> {
        OxdXml {
            version: String::from(OXD_VERSION),
            _phantom: PhantomData,
        }
    }
}

impl<A: StorageId, B: StorageId> ReplaceAsset<B> for OxdXml<A> {
    type From = A;

    type Output = OxdXml<B>;

    fn replace_asset<'a>(self, assets: &'a mut std::collections::HashMap<A, B>) -> Self::Output {
        OxdXml {
            version: self.version,
            _phantom: PhantomData,
        }
    }
}

impl<A: StorageId> GetAssets<A> for OxdXml<A> {
    fn get_assets(&self) -> Vec<A> {
        return vec![];
    }
}
