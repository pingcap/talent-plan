use kvs::{Result, KvStore};
use std::env::{self, current_exe, current_dir};
use std::fs::create_dir_all;
use std::ffi::OsStr;
use std::iter::empty;
use std::path::{PathBuf, Path};
use std::process::{Command, Output};

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

// `kvs get <KEY>` should print "Key not found" for an empty database and exit with zero.
#[test]
fn cli_get_nothing() {
    let temp_dir = temp_dir("cli_get_nothing");
    let output = run_with_dir_and_args(temp_dir, &["get", "key1"]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert!(dbg!(stdout).contains("Key not found"));
    assert!(output.status.success());
}

// `kvs set <KEY> <VALUE>` should print nothing and exit with zero.
#[test]
fn cli_set() {
    let temp_dir = temp_dir("cli_set");
    let output = run_with_dir_and_args(&temp_dir, &["set", "key1", "value1"]);
    assert!(output.stdout.is_empty());
    assert!(output.status.success());

    // run a second time
    let output = run_with_dir_and_args(&temp_dir, &["set", "key1", "value1"]);
    assert!(output.stdout.is_empty());
    assert!(output.status.success());
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
fn cli_invalid_subcommand() {
    assert!(!run_with_args(&["unknown", "subcommand"]).status.success());
}

// Should get previously stored value
#[test]
fn get_stored_value() -> Result<()> {
    let temp_dir = temp_dir("get_stored_value");
    let mut store = KvStore::open(&temp_dir)?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::open(&temp_dir)?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// Should overwrite existent value
#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = temp_dir("overwrite_value");
    let mut store = KvStore::open(&temp_dir)?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    store.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::open(&temp_dir)?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = temp_dir("get_non_existent_value");
    let mut store = KvStore::open(&temp_dir)?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    // Open from disk again and check persistent data
    drop(store);
    let store = KvStore::open(&temp_dir)?;
    assert_eq!(store.get("key2".to_owned())?, None);

    Ok(())
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

fn temp_dir(name: &str) -> PathBuf {
    let mut temp_dir = env::temp_dir();
    temp_dir.push(name);
    create_dir_all(&temp_dir).expect("unable to create working directory");
    temp_dir
}

fn run_with_args<I, S>(args: I) -> Output
    where
        I: IntoIterator<Item=S>,
        S: AsRef<OsStr>,
{
    run_with_dir_and_args(current_dir().expect("unable to get current_dir"), args)
}

fn run_with_dir_and_args<P, I, S>(dir: P, args: I) -> Output
    where
        P: AsRef<Path>,
        I: IntoIterator<Item=S>,
        S: AsRef<OsStr>,
{
    Command::new(binary_path())
        .current_dir(dir)
        .args(args)
        .output()
        .expect("failed to execute kvs binary")
}
