use crate::services::ContextKvStore;

use super::{Blob, KvStore};
use anyhow::{anyhow, Result};
use leveldb::batch::Batch;
use leveldb::database::batch::Writebatch;
use leveldb::database::cache::Cache;
use leveldb::database::Database;

use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

impl db_key::Key for Blob {
    fn from_u8(key: &[u8]) -> Self {
        Self(key.to_vec())
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.0)
    }
}

pub struct DiskKvStore(Database<Blob>);
impl DiskKvStore {
    pub fn new<P: AsRef<Path>>(path: P, cache_size: usize) -> Result<Self> {
        fs::create_dir_all(&path)?;
        let mut options = Options::new();
        options.create_if_missing = true;
        options.cache = Some(Cache::new(cache_size));
        Ok(Self(Database::open(path.as_ref(), options)?))
    }
}

impl ContextKvStore for DiskKvStore {}

impl KvStore for DiskKvStore {
    fn buffer(self) -> BTreeMap<Blob, Option<Blob>> {
        BTreeMap::new()
    }
    fn get_raw(&self, k: &Blob) -> Result<Option<Blob>> {
        let read_opts = ReadOptions::new();
        match self.0.get(read_opts, k) {
            Ok(v) => Ok(v.map(Blob)),
            Err(_) => Err(anyhow!("Database failure!")),
        }
    }
    fn batch_put_raw<I: Iterator<Item = (Blob, Option<Blob>)>>(&mut self, vals: I) -> Result<()> {
        let write_opts = WriteOptions::new();
        let mut batch = Writebatch::new();
        for op in vals {
            match op {
                (k, None) => batch.delete(k.clone()),
                (k, Some(v)) => batch.put(k.clone(), &v.0),
            }
        }
        match self.0.write(write_opts, &batch) {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Database failure!")),
        }
    }
}
