//! OXD File extension structure

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Id;

use crate::{asset::ReplaceAsset, storage::StorageId, OXD_VERSION};

#[derive(Clone, Serialize, Deserialize)]
pub struct OxdXml<A: StorageId> {
    pub id: Id,
    pub version: String,
    _phantom: PhantomData<A>,
}

impl<A: StorageId> OxdXml<A> {
    pub const TABLE: &str = "snapshots";

    pub fn new() -> OxdXml<A> {
        OxdXml { id: Id::String(String::new()), version: String::from(OXD_VERSION), _phantom: PhantomData }
    }

    pub fn clear_id(&mut self) {
        self.id = Id::String(String::new());
    }
}

impl<A: StorageId, B: StorageId> ReplaceAsset<B> for OxdXml<A> {
    type From = A;

    type Output = OxdXml<B>;

    fn replace_asset<'a>(self, assets: &'a mut std::collections::HashMap<A, B>) -> Self::Output {
        OxdXml {
            id: self.id,
            version: self.version,
            _phantom: PhantomData,
        }
    }
}
