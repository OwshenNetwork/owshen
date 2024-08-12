use crate::services::ContextKvStore;

use super::{Blob, KvStore};
use anyhow::Result;
use std::collections::BTreeMap;

pub struct MirrorKvStore<'a, K: ContextKvStore> {
    base: &'a K,
    overwrite: BTreeMap<Blob, Option<Blob>>,
}

impl<'a, K: ContextKvStore> MirrorKvStore<'a, K> {
    pub fn new(base: &'a K) -> Self {
        Self {
            base,
            overwrite: BTreeMap::new(),
        }
    }
    pub fn rollback(&self) -> Result<BTreeMap<Blob, Option<Blob>>> {
        let old_vals = self
            .overwrite
            .keys()
            .map(|k| self.base.get_raw(k))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.overwrite.keys().cloned().zip(old_vals).collect())
    }
}

impl<'a, K: ContextKvStore> ContextKvStore for MirrorKvStore<'a, K> {}

impl<'a, K: ContextKvStore> KvStore for MirrorKvStore<'a, K> {
    fn get_raw(&self, k: &Blob) -> Result<Option<Blob>> {
        if let Some(overwrite) = self.overwrite.get(k) {
            Ok(overwrite.clone())
        } else {
            self.base.get_raw(k)
        }
    }
    fn batch_put_raw<I: Iterator<Item = (Blob, Option<Blob>)>>(&mut self, vals: I) -> Result<()> {
        self.overwrite.extend(vals);
        Ok(())
    }
    fn buffer(self) -> BTreeMap<Blob, Option<Blob>> {
        self.overwrite
    }
}
