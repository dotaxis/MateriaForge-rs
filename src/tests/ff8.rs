//! Unit tests for `src/installers/ff8.rs`.
//!
//! Included via `#[path]` from that file, so this compiles as `crate::installers::ff8::tests` and can
//! reach its private items through `use super::*`.

use super::*;
use crate::installers::tests::detected;

fn steam_ff8() -> DetectedGame {
    detected(
        SupportedLaunchers::Steam,
        &format!("steam://rungameid/{FF8_APPID}"),
    )
}

fn unrelated_steam_game() -> DetectedGame {
    detected(SupportedLaunchers::Steam, "steam://rungameid/4000")
}

#[test]
fn detects_a_steam_install() {
    let installs = [unrelated_steam_game(), steam_ff8()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, Some(1));
    assert!(detection.is_detected());
}

#[test]
fn detects_nothing_without_ff8() {
    let installs = [unrelated_steam_game()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, None);
    assert!(!detection.is_detected());
}
