//! Unit tests for `src/installers/ff7.rs`.
//!
//! Included via `#[path]` from that file, so this compiles as `crate::installers::ff7::tests` and can
//! reach its private items through `use super::*`.

use super::*;
use crate::installers::tests::detected;

fn steam_ff7() -> DetectedGame {
    detected(
        SupportedLaunchers::Steam,
        &format!("steam://rungameid/{FF7_APPID}"),
    )
}

fn gog_ff7() -> DetectedGame {
    detected(
        SupportedLaunchers::HeroicGamesGOG,
        &format!("heroic://launch/gog/{FF7_GOG_APPID}"),
    )
}

fn unrelated_steam_game() -> DetectedGame {
    detected(SupportedLaunchers::Steam, "steam://rungameid/4000")
}

#[test]
fn detects_a_steam_install() {
    let installs = [unrelated_steam_game(), steam_ff7()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, Some(1));
    assert_eq!(detection.alt_index, None);
    assert!(detection.is_detected());
    // With a single install there is no prompt; this machine routes straight to it.
    assert_eq!(
        INSTALLER
            .choose_detected_index(&installs, detection)
            .unwrap(),
        Some(1)
    );
}

#[test]
fn detects_a_gog_install() {
    let installs = [unrelated_steam_game(), gog_ff7()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, None);
    assert_eq!(detection.alt_index, Some(1));
    // With a single install there is no prompt; this machine routes straight to it.
    assert_eq!(
        INSTALLER
            .choose_detected_index(&installs, detection)
            .unwrap(),
        Some(1)
    );
}

#[test]
fn detects_both_installs_independently() {
    let installs = [gog_ff7(), unrelated_steam_game(), steam_ff7()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, Some(2));
    assert_eq!(detection.alt_index, Some(0));
}

#[test]
fn detects_nothing_without_ff7() {
    let installs = [unrelated_steam_game()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, None);
    assert_eq!(detection.alt_index, None);
    assert!(!detection.is_detected());
}
