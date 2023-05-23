use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned, Deserialize};

use crate::{oxd::OxdXml, storage::StorageId};

#[derive(Serialize, Deserialize, Clone)]
pub enum AnyAction {
    RepositionObject(RepositionObject)
}

/// Action is a signal that contains data to change a oxd file and revert it back
pub trait Action<A: StorageId>: Serialize + DeserializeOwned + Send + Sync + Clone {
    type Error: Debug;

    /// Applying a change to oxd file
    fn redo(&self, oxd: &mut OxdXml<A>) -> Result<(), Self::Error>;

    /// Revert the action did to oxd file
    fn undo(&self, oxd: &mut OxdXml<A>) -> Result<(), Self::Error>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RepositionObject {
    screen: usize,
    object: usize,
}

impl <A: StorageId> Action<A> for RepositionObject {
    type Error = ();

    fn redo(&self, oxd: &mut OxdXml<A>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn undo(&self, oxd: &mut OxdXml<A>) -> Result<(), Self::Error> {
        Ok(())
    }
}
