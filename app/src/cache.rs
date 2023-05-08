use std::{
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    path::PathBuf,
    str::{from_utf8, Utf8Error}
};

use futures::StreamExt;
use surrealdb::{Connection, Surreal};
use tokio::io::{AsyncReadExt, AsyncBufRead};
use uuid::Uuid;

use crate::{
    asset::{detect_asset_type_by_ext, ReplaceAsset},
    oxd::OxdXml,
    storage::{Storage, StorageId},
};

use async_compression::tokio::bufread::XzDecoder;
use serde_xml_rs::de::from_str as xml_from_str;
use tokio_tar::Archive;

/// Cache that store contents of the oxd files
pub struct Cache<SI: StorageId, SE: Debug, T: Connection, S: Storage<SE, SI>> {
    db: Surreal<T>,
    storage: S,
    _phantom_se: PhantomData<SE>,
    _phantom_si: PhantomData<SI>,
}

#[derive(Debug)]
pub enum CacheFileError<SE: Debug> {
    Storage(SE),
    Io(std::io::Error),
    UnsupportedAsset(String),
    Serde(serde_xml_rs::Error),
    Utf8(Utf8Error),
    UnsupportedFile,
    Db(surrealdb::Error),
}

impl<SI: StorageId, SE: Debug, T: Connection, S: Storage<SE, SI>> Cache<SI, SE, T, S> {
    pub fn new(db: Surreal<T>, storage: S) -> Cache<SI, SE, T, S> {
        Cache {
            db,
            storage,
            _phantom_se: PhantomData,
            _phantom_si: PhantomData,
        }
    }

    /// Caching an oxd file
    pub async fn cache_file<R: AsyncBufRead + Unpin + Send + Sync>(
        &self,
        session_id: Uuid,
        buf: R,
    ) -> Result<(), CacheFileError<SE>> {
        let xz_decoder = XzDecoder::new(buf);
        let mut tar = Archive::new(xz_decoder);
        let mut entries = tar.entries().map_err(|e| CacheFileError::Io(e))?;
        let mut oxd_content_opt: Option<OxdXml<PathBuf>> = None;
        let mut path_id_map: HashMap<PathBuf, SI> = HashMap::new();
        while let Some(entry) = entries.next().await {
            let mut entry = entry.map_err(|e| CacheFileError::Io(e))?;
            let path = entry.path().map_err(|e| CacheFileError::Io(e))?;
            match path.extension() {
                Some(ext) => {
                    if ext == "oxd" {
                        let mut xml_bytes: Vec<u8> = Vec::new();
                        entry
                            .read_to_end(&mut xml_bytes)
                            .await
                            .map_err(|e| CacheFileError::Io(e))?;
                        let xml_str = from_utf8(&xml_bytes).map_err(|e| CacheFileError::Utf8(e))?;
                        let xml: OxdXml<PathBuf> =
                            xml_from_str(xml_str).map_err(|e| CacheFileError::Serde(e))?;
                        oxd_content_opt = Some(xml);
                    } else {
                        match detect_asset_type_by_ext(ext.to_str().unwrap()) {
                            Some(_) => {
                                let ext = ext.to_str().unwrap();
                                let ext_cloned = String::from(ext);
                                let path_new = entry.path().unwrap();
                                let path_buf = path_new.to_path_buf();
                                let id = self
                                    .storage
                                    .put(&mut entry, format!("session/{}/assets", session_id), ext_cloned)
                                    .await
                                    .map_err(|e| CacheFileError::Storage(e))?;
                                path_id_map.insert(path_buf, id);
                            }
                            None => {
                                return Err(CacheFileError::UnsupportedAsset(
                                    path.to_str().unwrap().into(),
                                ));
                            }
                        }
                    }
                }
                None => {
                    return Err(CacheFileError::UnsupportedAsset(
                        path.to_str().unwrap().into(),
                    ));
                }
            }
        }

        match oxd_content_opt.take() {
            Some(oxd_content) => {
                let replaced_oxd: OxdXml<SI> = oxd_content.replace_asset(&mut path_id_map);

                for (_, v) in path_id_map {
                    self.storage
                        .delete(v)
                        .await
                        .map_err(|e| CacheFileError::Storage(e))?;
                }

                let _created_content: OxdXml<SI> = self
                    .db
                    .create(("oxd", session_id.to_string()))
                    .content(replaced_oxd)
                    .await
                    .map_err(|e| CacheFileError::Db(e))?;
            }
            None => {
                return Err(CacheFileError::UnsupportedFile);
            }
        }

        Ok(())
    }
}
