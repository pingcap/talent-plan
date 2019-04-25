use kvs::{KvStore, Result};
use std::collections::HashMap;
use std::env::{current_dir, current_exe};
use std::ffi::OsStr;
use std::fs::Metadata;
use std::io;
use std::iter::empty;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;
use walkdir::{DirEntry, WalkDir};

// `kvs` with no args should exit with a non-zero code.
#[test]
fn cli_no_args() {
    let output = run_with_args(empty::<&OsStr>());
    assert!(!output.status.success())
}

// `kvs -V` should print the version
#[test]
fn cli_version() {
    let output = run_with_args(&["-V"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

// `kvs get <KEY>` should print "Key not found" for a non-existent key and exit with zero.
#[test]
fn cli_get_non_existent_key() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let output = run_with_dir_and_args(temp_dir.path(), &["get", "key1"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "Key not found");
    assert!(output.status.success());
}

// `kvs rm <KEY>` should print "Key not found" for an empty database and exit with zero.
#[test]
fn cli_rm_non_existent_key() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let output = run_with_dir_and_args(temp_dir.path(), &["rm", "key1"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "Key not found");
    assert!(!output.status.success());
}

// `kvs set <KEY> <VALUE>` should print nothing and exit with zero.
#[test]
fn cli_set() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let output = run_with_dir_and_args(temp_dir.path(), &["set", "key1", "value1"]);
    assert!(output.stdout.is_empty());
    assert!(output.status.success());

    // run a second time
    let output = run_with_dir_and_args(temp_dir.path(), &["set", "key1", "value1"]);
    assert!(output.stdout.is_empty());
    assert!(output.status.success());
}

#[test]
fn cli_get_stored() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");

    let mut store = KvStore::open(temp_dir.path())?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;
    drop(store);

    let output = run_with_dir_and_args(temp_dir.path(), &["get", "key1"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "value1");
    assert!(output.status.success());

    let output = run_with_dir_and_args(temp_dir.path(), &["get", "key2"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "value2");
    assert!(output.status.success());

    Ok(())
}

// `kvs rm <KEY>` should print nothing and exit with zero.
#[test]
fn cli_rm_stored() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");

    let mut store = KvStore::open(temp_dir.path())?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    drop(store);

    let output = run_with_dir_and_args(temp_dir.path(), &["rm", "key1"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert!(stdout.is_empty());
    assert!(output.status.success());

    let output = run_with_dir_and_args(temp_dir.path(), &["get", "key1"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "Key not found");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn cli_invalid_get() {
    assert!(!run_with_args(&["get"]).status.success());
    assert!(!run_with_args(&["get", "extra", "field"]).status.success());
}

#[test]
fn cli_invalid_set() {
    assert!(!run_with_args(&["set"]).status.success());
    assert!(!run_with_args(&["set", "missing_field"]).status.success());
    assert!(!run_with_args(&["set", "extra", "extra", "field"])
        .status
        .success());
}

#[test]
fn cli_invalid_rm() {
    assert!(!run_with_args(&["rm"]).status.success());
    assert!(!run_with_args(&["rm", "extra", "field"]).status.success());
}

#[test]
fn cli_invalid_subcommand() {
    assert!(!run_with_args(&["unknown", "subcommand"]).status.success());
}

// Should get previously stored value
#[test]
fn get_stored_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let mut store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// Should overwrite existent value
#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    store.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let mut store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));
    store.set("key1".to_owned(), "value3".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value3".to_owned()));

    Ok(())
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    // Open from disk again and check persistent data
    drop(store);
    let mut store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    Ok(())
}

#[test]
fn remove_non_existent_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;
    assert!(store.remove("key1".to_owned()).is_err());
    Ok(())
}

#[test]
fn remove_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    assert!(store.remove("key1".to_owned()).is_ok());
    assert_eq!(store.get("key1".to_owned())?, None);
    Ok(())
}

// Insert data until total size of the directory decreases.
// Test data correctness after compaction.
#[test]
fn compaction() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

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
        let mut store = KvStore::open(temp_dir.path())?;
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            assert_eq!(store.get(key)?, Some(format!("{}", iter)));
        }
        return Ok(());
    }

    panic!("No compaction detected");
}

// Path to kvs binary
fn binary_path() -> PathBuf {
    // Path to cargo executables
    // Adapted from https://github.com/rust-lang/cargo/blob/485670b3983b52289a2f353d589c57fae2f60f82/tests/testsuite/support/mod.rs#L507
    let mut path = current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .unwrap();

    path.push("kvs");
    path
}

fn run_with_args<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_with_dir_and_args(current_dir().expect("unable to get current_dir"), args)
}

fn run_with_dir_and_args<P, I, S>(dir: P, args: I) -> Output
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(binary_path())
        .current_dir(dir)
        .args(args)
        .output()
        .expect("failed to execute kvs binary")
}
