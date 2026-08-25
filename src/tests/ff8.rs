//! Unit tests for `src/installers/ff8.rs`.
//!
//! Included via `#[path]` from that file, so this compiles as `crate::installers::ff8::tests` and can
//! reach its private items through `use super::*`.

use super::*;
use crate::gamelib_helper::{Game, PrefixRunner, Runner};
use crate::installers::tests::detected;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct FakeGame {
    app_id: u32,
    path: PathBuf,
    prefix: PathBuf,
}

impl Game for FakeGame {
    fn app_id(&self) -> u32 {
        self.app_id
    }

    fn name(&self) -> &str {
        "Fake FF8"
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn prefix(&self) -> &Path {
        &self.prefix
    }

    fn runner(&self) -> Option<&Runner> {
        None
    }
}

impl PrefixRunner for FakeGame {
    fn run_in_prefix(&self, _exe_to_launch: PathBuf, _args: Option<Vec<String>>) -> Result<()> {
        Ok(())
    }
}

fn steam_ff8() -> DetectedGame {
    detected(
        SupportedLaunchers::Steam,
        &format!("steam://rungameid/{FF8_APPID}"),
    )
}

fn steam_ff8_remastered() -> DetectedGame {
    detected(
        SupportedLaunchers::Steam,
        &format!("steam://rungameid/{FF8_REMASTERED_APPID}"),
    )
}

fn gog_ff8() -> DetectedGame {
    detected(
        SupportedLaunchers::HeroicGamesGOG,
        &format!("heroic://launch/gog/{FF8_GOG_APPID}"),
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
    assert_eq!(detection.alt_index, None);
    assert!(detection.is_detected());
}

#[test]
fn detects_a_remastered_steam_install() {
    let installs = [unrelated_steam_game(), steam_ff8_remastered()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, Some(1));
    assert_eq!(detection.alt_index, None);
    assert!(detection.is_detected());
}

#[test]
fn detects_a_gog_install() {
    let installs = [unrelated_steam_game(), gog_ff8()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, None);
    assert_eq!(detection.alt_index, Some(1));
    assert!(detection.is_detected());
}

#[test]
fn detects_nothing_without_ff8() {
    let installs = [unrelated_steam_game()];

    let detection = INSTALLER.detect(&installs);

    assert_eq!(detection.steam_index, None);
    assert_eq!(detection.alt_index, None);
    assert!(!detection.is_detected());
}

#[test]
fn creates_remastered_steam_save_slot() {
    let tmp = TempDir::new().unwrap();
    let game = FakeGame {
        app_id: FF8_REMASTERED_APPID,
        path: tmp.path().join("game"),
        prefix: tmp.path().join("prefix"),
    };

    ensure_ff8_user_slot(&game).unwrap();

    assert!(
        game.prefix
            .join("drive_c/users/steamuser/Documents/My Games/FINAL FANTASY VIII Remastered/Steam/0/game_data/user/saves")
            .is_dir()
    );
}

#[test]
fn creates_remastered_gog_save_slot() {
    let tmp = TempDir::new().unwrap();
    let game = FakeGame {
        app_id: FF8_GOG_APPID,
        path: tmp.path().join("game"),
        prefix: tmp.path().join("prefix"),
    };

    ensure_ff8_user_slot(&game).unwrap();

    assert!(
        game.prefix
            .join("drive_c/users/steamuser/Documents/My Games/FINAL FANTASY VIII Remastered/GOG/0/game_data/user/saves")
            .is_dir()
    );
}
