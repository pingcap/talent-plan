use kvs::thread_pool::RayonThreadPool;
use kvs::{KvStore, KvsEngine, KvsError, Result};
use tempfile::TempDir;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use walkdir::WalkDir;

// Should get previously stored value
#[test]
fn get_stored_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    store.set("key1".to_owned(), "value1".to_owned()).wait()?;
    store.set("key2".to_owned(), "value2".to_owned()).wait()?;

    assert_eq!(
        store.get("key1".to_owned()).wait()?,
        Some("value1".to_owned())
    );
    assert_eq!(
        store.get("key2".to_owned()).wait()?,
        Some("value2".to_owned())
    );

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
    assert_eq!(
        store.get("key1".to_owned()).wait()?,
        Some("value1".to_owned())
    );
    assert_eq!(
        store.get("key2".to_owned()).wait()?,
        Some("value2".to_owned())
    );

    Ok(())
}

// Should overwrite existent value
#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    store.set("key1".to_owned(), "value1".to_owned()).wait()?;
    assert_eq!(
        store.get("key1".to_owned()).wait()?,
        Some("value1".to_owned())
    );
    store.set("key1".to_owned(), "value2".to_owned()).wait()?;
    assert_eq!(
        store.get("key1".to_owned()).wait()?,
        Some("value2".to_owned())
    );

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
    assert_eq!(
        store.get("key1".to_owned()).wait()?,
        Some("value2".to_owned())
    );
    store.set("key1".to_owned(), "value3".to_owned()).wait()?;
    assert_eq!(
        store.get("key1".to_owned()).wait()?,
        Some("value3".to_owned())
    );

    Ok(())
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    store.set("key1".to_owned(), "value1".to_owned()).wait()?;
    assert_eq!(store.get("key2".to_owned()).wait()?, None);

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
    assert_eq!(store.get("key2".to_owned()).wait()?, None);

    Ok(())
}

#[test]
fn remove_non_existent_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
    assert!(store.remove("key1".to_owned()).wait().is_err());
    Ok(())
}

#[test]
fn remove_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
    store.set("key1".to_owned(), "value1".to_owned()).wait()?;
    assert!(store.remove("key1".to_owned()).wait().is_ok());
    assert_eq!(store.get("key1".to_owned()).wait()?, None);
    Ok(())
}

// Insert data until total size of the directory decreases.
// Test data correctness after compaction.
#[test]
fn compaction() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    let dir_size = || {
        let entries = WalkDir::new(temp_dir.path()).into_iter();
        let len: walkdir::Result<u64> = entries
            .map(|res| {
                res.and_then(|entry| entry.metadata())
                    .map(|metadata| metadata.len())
            })
            .sum();
        len.expect("fail to get directory size")
    };

    let mut current_size = dir_size();
    for iter in 0..1000 {
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            let value = format!("{}", iter);
            store.set(key, value).wait()?;
        }

        let new_size = dir_size();
        if new_size > current_size {
            current_size = new_size;
            continue;
        }
        // Compaction triggered

        drop(store);
        // reopen and check content
        let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            assert_eq!(store.get(key).wait()?, Some(format!("{}", iter)));
        }
        return Ok(());
    }

    panic!("No compaction detected");
}

#[test]
fn concurrent_set() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");

    // concurrent set in 8 threads
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 8)?;
    let runtime = Runtime::new().unwrap();
    let executor = runtime.executor();
    runtime.block_on_all(future::lazy(move || {
        for i in 0..10000 {
            executor.spawn(
                store
                    .set(format!("key{}", i), format!("value{}", i))
                    .map_err(|_| ()),
            );
        }
        future::ok::<(), KvsError>(())
    }))?;

    // We only check concurrent set in this test, so we check sequentially here
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
    for i in 0..10000 {
        assert_eq!(
            store.get(format!("key{}", i)).wait()?,
            Some(format!("value{}", i))
        );
    }

    Ok(())
}

#[test]
fn concurrent_get() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 8)?;
    // We only check concurrent get in this test, so we set sequentially here
    for i in 0..100 {
        store
            .set(format!("key{}", i), format!("value{}", i))
            .wait()
            .unwrap();
    }

    let runtime = Runtime::new().unwrap();
    let executor = runtime.executor();
    runtime.block_on_all(future::lazy(move || {
        for thread_id in 0..100 {
            for i in 0..100 {
                let key_id = (i + thread_id) % 100;
                executor.spawn(
                    store
                        .get(format!("key{}", key_id))
                        .map(move |res| {
                            assert_eq!(res, Some(format!("value{}", key_id)));
                        })
                        .map_err(|_| ()),
                );
            }
        }
        future::ok::<(), KvsError>(())
    }))?;

    // reload from disk and test again
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 8)?;
    let runtime = Runtime::new().unwrap();
    let executor = runtime.executor();
    runtime.block_on_all(future::lazy(move || {
        for thread_id in 0..100 {
            for i in 0..100 {
                let key_id = (i + thread_id) % 100;
                executor.spawn(
                    store
                        .get(format!("key{}", key_id))
                        .map(move |res| {
                            assert_eq!(res, Some(format!("value{}", key_id)));
                        })
                        .map_err(|_| ()),
                );
            }
        }
        future::ok::<(), KvsError>(())
    }))?;

    Ok(())
}
