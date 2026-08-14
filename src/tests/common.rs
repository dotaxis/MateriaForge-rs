//! Unit tests for `src/installers/common.rs`.
//!
//! Included via `#[path]` from that file, so this compiles as `crate::installers::common::tests` and can
//! reach its private items through `use super::*`.

use super::*;
use tempfile::TempDir;

/// Creates `drive_c/users/<name>/Documents` inside a fake Wine prefix.
fn add_user_with_documents(prefix: &Path, name: &str) {
    fs::create_dir_all(prefix.join("drive_c/users").join(name).join("Documents")).unwrap();
}

#[test]
fn finds_the_documents_dir_in_a_proton_prefix() {
    // Proton creates its prefixes with a `steamuser` account; both call sites
    // (FF7 2013 and FF8) are Steam-only, so this is the path that actually runs.
    let tmp = TempDir::new().unwrap();
    add_user_with_documents(tmp.path(), "steamuser");

    assert_eq!(
        wine_user_documents_dir(tmp.path()),
        tmp.path().join("drive_c/users/steamuser/Documents")
    );
}

#[test]
fn falls_back_to_steamuser_in_a_fresh_prefix() {
    // A prefix the game has never launched in has no Documents dir yet. The returned path is
    // then handed to `ensure_wine_user_slot`, which creates it, so it has to be the right one.
    let tmp = TempDir::new().unwrap();

    assert_eq!(
        wine_user_documents_dir(tmp.path()),
        tmp.path().join("drive_c/users/steamuser/Documents")
    );
}

#[test]
fn creates_the_default_user_slot_in_an_empty_base() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join("Square Enix/FINAL FANTASY VII Steam");

    let slot = ensure_wine_user_slot(&base, "user_12345678").unwrap();

    assert_eq!(slot, base.join("user_12345678"));
    assert!(slot.is_dir());
}

#[test]
fn reuses_an_existing_user_slot() {
    // The game writes saves into whichever user_* directory already exists, so creating a
    // second one would strand them.
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join("base");
    let existing = base.join("user_87654321");
    fs::create_dir_all(&existing).unwrap();

    let slot = ensure_wine_user_slot(&base, "user_12345678").unwrap();

    assert_eq!(slot, existing);
    assert!(!base.join("user_12345678").exists());
}

#[test]
fn ignores_directories_that_are_not_user_slots() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join("base");
    fs::create_dir_all(base.join("metadata")).unwrap();
    fs::write(base.join("user_notadir"), "").unwrap();

    let slot = ensure_wine_user_slot(&base, "user_12345678").unwrap();

    assert_eq!(slot, base.join("user_12345678"));
    assert!(slot.is_dir());
}
