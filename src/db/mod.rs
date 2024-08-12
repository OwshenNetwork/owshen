mod disk;
mod key;
mod mirror;
mod ram;
mod value;

pub use disk::DiskKvStore;
pub use key::Key;
pub use mirror::MirrorKvStore;
pub use ram::RamKvStore;

pub use value::Value;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash)]
pub struct Blob(Vec<u8>);

// TODO‌: Range Search
// self.db.range(range)

pub trait KvStore {
    fn get(&self, k: Key) -> Result<Option<Value>> {
        Ok(if let Some(b) = self.get_raw(&k.try_into()?)? {
            Some(bincode::deserialize(&b.0)?)
        } else {
            None
        })
    }
    fn put(&mut self, k: Key, v: Option<Value>) -> Result<()> {
        self.batch_put([(k, v)].into_iter())
    }
    fn batch_put<I: Iterator<Item = (Key, Option<Value>)>>(&mut self, vals: I) -> Result<()> {
        let mut conv = Vec::new();
        for (k, v) in vals {
            conv.push((
                k.try_into()?,
                if let Some(v) = v {
                    Some(v.try_into()?)
                } else {
                    None
                },
            ));
        }
        self.batch_put_raw(conv.into_iter())
    }
    fn get_raw(&self, k: &Blob) -> Result<Option<Blob>>;
    fn batch_put_raw<I: Iterator<Item = (Blob, Option<Blob>)>>(&mut self, vals: I) -> Result<()>;
    fn buffer(self) -> BTreeMap<Blob, Option<Blob>>;
}
