//! OXD File extension structure

use std::marker::PhantomData;

use serde::{Serialize, Deserialize};
use surrealdb::sql::Id;

use crate::{asset::ReplaceAsset, storage::StorageId};

#[derive(Clone, Serialize, Deserialize)]
pub struct OxdXml<A: StorageId> {
    id: Id,
    version: String,
    _phantom: PhantomData<A>
}

impl <A: StorageId> OxdXml<A> {
    pub const TABLE: &str = "snapshots";

    pub fn clear_id(&mut self) {
        self.id = Id::String(String::new());
    }
}

impl <A: StorageId, B: StorageId> ReplaceAsset<B> for OxdXml<A> {
    type From = A;

    type Output = OxdXml<B>;

    fn replace_asset<'a>(self, assets: &'a mut std::collections::HashMap<A, B>)-> Self::Output {
        OxdXml {
            id: self.id,
            version: self.version,
            _phantom: PhantomData
        }
    }
}
