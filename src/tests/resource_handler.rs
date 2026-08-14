//! Unit tests for `src/resource_handler.rs`.
//!
//! Included via `#[path]` from that file, so this compiles as `crate::resource_handler::tests` and can
//! reach its private items through `use super::*`.

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn write_creates_missing_parent_directories() {
    let tmp = TempDir::new().unwrap();
    let file = as_str(
        "dxvk.conf".to_string(),
        tmp.path().join("deeply/nested/dir"),
        "dxvk.enableAsync = True",
    );

    file.write().unwrap();

    // Asserts the explicit path rather than reading `file.destination` back, pinning that the
    // constructor joins the name onto the directory — callers hand `.destination` straight to
    // things like xdg-icon-resource, so that path has to be the full file path.
    assert_eq!(
        fs::read_to_string(tmp.path().join("deeply/nested/dir/dxvk.conf")).unwrap(),
        "dxvk.enableAsync = True"
    );
}

#[test]
fn write_overwrites_existing_contents() {
    let tmp = TempDir::new().unwrap();
    let file = as_str("dxvk.conf".to_string(), tmp.path().to_path_buf(), "new");
    fs::write(&file.destination, "old").unwrap();

    file.write().unwrap();

    assert_eq!(fs::read_to_string(&file.destination).unwrap(), "new");
}

#[test]
fn desktop_copy_works_without_an_existing_desktop_dir() {
    // Configuration: minimal WMs/distros often ship no ~/Desktop at all. The Desktop
    // shortcut copy must create it rather than fail.
    let tmp = TempDir::new().unwrap();
    let file = as_str(
        "7th Heaven (2013).desktop".to_string(),
        tmp.path().join("applications"),
        "[Desktop Entry]",
    );
    let desktop = tmp.path().join("Desktop/7th Heaven (2013).desktop");

    file.write_to(&desktop).unwrap();

    assert_eq!(fs::read_to_string(&desktop).unwrap(), "[Desktop Entry]");
}

#[test]
fn write_if_missing_creates_the_file_once() {
    let tmp = TempDir::new().unwrap();
    let file = as_bytes(
        "timeout.exe".to_string(),
        tmp.path().join("drive_c/windows/system32"),
        b"replacement",
    );

    assert!(file.write_if_missing().unwrap());
    assert_eq!(
        fs::read(tmp.path().join("drive_c/windows/system32/timeout.exe")).unwrap(),
        b"replacement"
    );
}

#[test]
fn write_if_missing_leaves_an_existing_file_alone() {
    // The bundled timeout.exe must never clobber one the prefix already has.
    let tmp = TempDir::new().unwrap();
    let file = as_bytes(
        "timeout.exe".to_string(),
        tmp.path().to_path_buf(),
        b"replacement",
    );
    fs::write(&file.destination, b"original").unwrap();

    assert!(!file.write_if_missing().unwrap());
    assert_eq!(fs::read(&file.destination).unwrap(), b"original");
}
