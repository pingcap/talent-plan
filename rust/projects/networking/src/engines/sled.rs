use crate::{KvsEngine, Result};
use sled::{Db, Tree};

impl KvsEngine for Db {
    fn set(&self, key: String, value: String) -> Result<()> {
        let tree: &Tree = &*self;
        Ok(tree.set(key, value.into_bytes()).map(|_| ())?)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let tree: &Tree = &*self;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }
}
