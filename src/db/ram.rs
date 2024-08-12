use crate::services::ContextKvStore;

use super::{Blob, KvStore};
use anyhow::Result;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct RamKvStore {
    db: BTreeMap<Blob, Blob>,
}

impl ContextKvStore for RamKvStore {}

impl KvStore for RamKvStore {
    fn get_raw(&self, k: &Blob) -> Result<Option<Blob>> {
        Ok(self.db.get(k).cloned())
    }
    fn batch_put_raw<I: Iterator<Item = (Blob, Option<Blob>)>>(&mut self, vals: I) -> Result<()> {
        for (k, v) in vals {
            if let Some(v) = v {
                self.db.insert(k, v);
            } else {
                self.db.remove(&k);
            }
        }
        Ok(())
    }
    fn buffer(self) -> BTreeMap<Blob, Option<Blob>> {
        BTreeMap::new()
    }
}

impl RamKvStore {
    pub fn new() -> Self {
        Self {
            db: BTreeMap::new(),
        }
    }
}
