//! Shared test fixtures for `src/installers/mod.rs`.
//!
//! Included via `#[path]` from that file, so this compiles as `crate::installers::tests` and is
//! reachable from the `ff7` and `ff8` test modules nested beneath it.

use lib_game_detector::data::{Game as DetectedGame, SupportedLaunchers};

/// Builds a detected game whose launch command carries `arg`.
///
/// `lib_game_detector` emits the game URL as a standalone argument — `steam://rungameid/<id>`
/// for Steam, `heroic://launch/gog/<id>` for Heroic — so that is the shape callers pass here.
/// Only `source` and `launch_command` matter to the code under test.
pub(crate) fn detected(source: SupportedLaunchers, arg: &str) -> DetectedGame {
    let mut launch_command = std::process::Command::new("sh");
    launch_command.arg(arg);

    DetectedGame {
        title: "test game".to_string(),
        path_icon: None,
        path_box_art: None,
        path_game_dir: None,
        launch_command,
        source,
    }
}
