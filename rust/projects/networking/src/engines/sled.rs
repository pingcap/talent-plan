use crate::{KvsEngine, KvsError, Result};
use sled::{Db, Tree};

impl KvsEngine for Db {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let tree: &Tree = &*self;
        tree.set(key, value.into_bytes()).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let tree: &Tree = &*self;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let tree: &Tree = &*self;
        tree.del(key)?.ok_or(KvsError::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}
