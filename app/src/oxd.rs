//! OXD File extension structure

use std::marker::PhantomData;

use serde::{Serialize, Deserialize};

use crate::{asset::ReplaceAsset, storage::StorageId};

#[derive(Clone, Serialize, Deserialize)]
pub struct OxdXml<A: StorageId> {
    version: String,
    _phantom: PhantomData<A>
}

impl <A: StorageId, B: StorageId> ReplaceAsset<B> for OxdXml<A> {
    type From = A;

    type Output = OxdXml<B>;

    fn replace_asset<'a>(self, assets: &'a mut std::collections::HashMap<A, B>)-> Self::Output {
        OxdXml {
            version: self.version,
            _phantom: PhantomData
        }
    }
}
