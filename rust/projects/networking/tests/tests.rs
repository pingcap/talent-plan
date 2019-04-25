use assert_cmd::prelude::*;
use kvs::{KvStore, KvsEngine, Result};
use predicates::str::{contains, is_empty};
use std::fs::{self, File};
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use tempfile::TempDir;
use walkdir::WalkDir;

// `kvs-client` with no args should exit with a non-zero code.
#[test]
fn client_cli_no_args() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-client").unwrap();
    cmd.current_dir(&temp_dir).assert().failure();
}

#[test]
fn client_cli_invalid_get() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "extra", "field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_set() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "missing_field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "key", "value", "extra_field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "key", "value", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_subcommand() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["unknown"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

// `kvs-client -V` should print the version
#[test]
fn client_cli_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-client").unwrap();
    cmd.args(&["-V"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

// `kvs-server -V` should print the version
#[test]
fn server_cli_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-server").unwrap();
    cmd.args(&["-V"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_log_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let stderr_path = temp_dir.path().join("stderr");
    let mut cmd = Command::cargo_bin("kvs-server").unwrap();
    let mut child = cmd
        .args(&["--engine", "kvs", "--addr", "127.0.0.1:4001"])
        .current_dir(&temp_dir)
        .stderr(File::create(&stderr_path).unwrap())
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_secs(1));
    child.kill().expect("server exited before killed");

    let content = fs::read_to_string(&stderr_path).expect("unable to read from stderr file");
    assert!(content.contains(env!("CARGO_PKG_VERSION")));
    assert!(content.contains("kvs"));
    assert!(content.contains("127.0.0.1:4001"));
}

#[test]
fn cli_wrong_engine() {
    // sled first, kvs second
    {
        let temp_dir = TempDir::new().unwrap();
        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        let mut child = cmd
            .args(&["--engine", "sled", "--addr", "127.0.0.1:4002"])
            .current_dir(&temp_dir)
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        child.kill().expect("server exited before killed");

        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        cmd.args(&["--engine", "kvs", "--addr", "127.0.0.1:4003"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }

    // kvs first, sled second
    {
        let temp_dir = TempDir::new().unwrap();
        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        let mut child = cmd
            .args(&["--engine", "kvs", "--addr", "127.0.0.1:4002"])
            .current_dir(&temp_dir)
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        child.kill().expect("server exited before killed");

        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        cmd.args(&["--engine", "sled", "--addr", "127.0.0.1:4003"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }
}

fn cli_access_server(engine: &str, addr: &str) {
    let (sender, receiver) = mpsc::sync_channel(0);
    let temp_dir = TempDir::new().unwrap();
    let mut server = Command::cargo_bin("kvs-server").unwrap();
    let mut child = server
        .args(&["--engine", engine, "--addr", addr])
        .current_dir(&temp_dir)
        .spawn()
        .unwrap();
    let handle = thread::spawn(move || {
        let _ = receiver.recv(); // wait for main thread to finish
        child.kill().expect("server exited before killed");
    });
    thread::sleep(Duration::from_secs(1));

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "key1", "value1", "--addr", addr])
        .current_dir(&temp_dir)
        .assert()
        .success().stdout(is_empty());

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key1", "--addr", addr])
        .current_dir(&temp_dir)
        .assert()
        .success().stdout("value1\n");

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["set", "key1", "value2", "--addr", addr])
        .current_dir(&temp_dir)
        .assert()
        .success().stdout(is_empty());

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key1", "--addr", addr])
        .current_dir(&temp_dir)
        .assert()
        .success().stdout("value2\n");

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key2", "--addr", addr])
        .current_dir(&temp_dir)
        .assert()
        .success().stdout(contains("Key not found"));

    sender.send(()).unwrap();
    handle.join().unwrap();

    // Reopen and check value
    let (sender, receiver) = mpsc::sync_channel(0);
    let mut server = Command::cargo_bin("kvs-server").unwrap();
    let mut child = server
        .args(&["--engine", engine, "--addr", addr])
        .current_dir(&temp_dir)
        .spawn()
        .unwrap();
    let handle = thread::spawn(move || {
        let _ = receiver.recv(); // wait for main thread to finish
        child.kill().expect("server exited before killed");
    });
    thread::sleep(Duration::from_secs(1));

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(&["get", "key2", "--addr", addr])
        .current_dir(&temp_dir)
        .assert()
        .success().stdout(contains("Key not found"));
    sender.send(()).unwrap();
    handle.join().unwrap();
}

#[test]
fn cli_access_server_kvs_engine() {
    cli_access_server("kvs", "127.0.0.1:4004");
}

#[test]
fn cli_access_server_sled_engine() {
    cli_access_server("sled", "127.0.0.1:4005");
}

// Should get previously stored value
#[test]
fn get_stored_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// Should overwrite existent value
#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    store.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));
    store.set("key1".to_owned(), "value3".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value3".to_owned()));

    Ok(())
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    Ok(())
}

// Insert data until total size of the directory decreases.
// Test data correctness after compaction.
#[test]
fn compaction() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

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
            store.set(key, value)?;
        }

        let new_size = dir_size();
        if new_size > current_size {
            current_size = new_size;
            continue;
        }
        // Compaction triggered

        drop(store);
        // reopen and check content
        let store = KvStore::open(temp_dir.path())?;
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            assert_eq!(store.get(key)?, Some(format!("{}", iter)));
        }
        return Ok(());
    }

    panic!("No compaction detected");
}
