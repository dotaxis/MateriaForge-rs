//! Unit tests for `src/gamelib_helper/steam_proton.rs`.
//!
//! Included via `#[path]` from that file, so this compiles as `crate::gamelib_helper::steam_proton::tests` and can
//! reach its private items through `use super::*`.

use super::*;
use std::collections::BTreeSet;
use std::path::Path;
use tempfile::TempDir;

/// A synthesized Steam installation on disk.
///
/// `SteamDir::from_dir` only checks that the path is a directory, and the parsers need far
/// less than a real install writes out: `libraryfolders.vdf` needs a numbered entry with a
/// `path`, and an app manifest needs `appid`, `name`, and `installdir`. Everything else on
/// `steamlocate::app::App` is optional.
struct FakeSteam {
    _tmp: TempDir,
    root: PathBuf,
}

impl FakeSteam {
    fn new() -> Self {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let root = tmp.path().join("Steam");
        fs::create_dir_all(root.join("steamapps/common")).unwrap();
        fs::write(
            root.join("steamapps/libraryfolders.vdf"),
            format!(
                "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n",
                root.display()
            ),
        )
        .unwrap();

        Self { _tmp: tmp, root }
    }

    /// Installs an app and returns its resolved install directory.
    fn app(&self, id: u32, name: &str, install_dir: &str) -> PathBuf {
        fs::write(
            self.root.join(format!("steamapps/appmanifest_{id}.acf")),
            format!(
                "\"AppState\"\n{{\n\t\"appid\"\t\t\"{id}\"\n\t\"name\"\t\t\"{name}\"\n\t\"installdir\"\t\t\"{install_dir}\"\n}}\n"
            ),
        )
        .unwrap();

        let app_dir = self.root.join("steamapps/common").join(install_dir);
        fs::create_dir_all(&app_dir).unwrap();
        app_dir
    }

    /// Installs an app that carries a `proton` binary, i.e. one that counts as a runner.
    fn proton_app(&self, id: u32, name: &str, install_dir: &str) -> PathBuf {
        let app_dir = self.app(id, name, install_dir);
        fs::write(app_dir.join("proton"), "#!/bin/sh\n").unwrap();
        app_dir
    }

    /// Points a runner at a required Steam Linux Runtime.
    fn toolmanifest(&self, app_dir: &Path, require_tool_appid: u32) {
        fs::write(
            app_dir.join("toolmanifest.vdf"),
            format!("\"manifest\"\n{{\n\t\"require_tool_appid\"\t\t\"{require_tool_appid}\"\n}}\n"),
        )
        .unwrap();
    }

    /// Creates a custom runner directory under a `compatibilitytools.d`-style parent.
    fn compat_tool(&self, compat_dir: &Path, name: &str) -> PathBuf {
        let dir = compat_dir.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("proton"), "#!/bin/sh\n").unwrap();
        dir
    }

    /// A `compatibilitytools.d` directory inside the Steam dir itself.
    fn steam_compat_dir(&self) -> PathBuf {
        let dir = self.root.join("compatibilitytools.d");
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn steam_dir(&self) -> steamlocate::SteamDir {
        steamlocate::SteamDir::from_dir(&self.root).expect("failed to open fake steam dir")
    }
}

fn names(runners: &[Runner]) -> BTreeSet<&str> {
    runners.iter().map(|r| r.pretty_name.as_str()).collect()
}

// Every test passes `compat_dirs` explicitly so nothing reads the real
// /usr/share/steam/compatibilitytools.d, which is populated on some dev machines.
const NO_COMPAT_DIRS: &[PathBuf] = &[];

#[test]
fn finds_proton_app_from_library() {
    let steam = FakeSteam::new();
    steam.proton_app(2_805_730, "Proton 9.0", "Proton 9.0");

    let runners = find_all_versions_in(&steam.steam_dir(), NO_COMPAT_DIRS).unwrap();

    assert_eq!(runners.len(), 1);
    assert_eq!(runners[0].pretty_name, "Proton 9.0");
    // Derived by lowercasing, truncating at the first '.', then underscoring spaces.
    assert_eq!(runners[0].name, "proton_9");
    assert!(runners[0].path.ends_with("proton"));
    assert!(runners[0].runtime.is_none());
    // Library-installed Protons are official; "Automatic" may pick them.
    assert!(!runners[0].is_custom);
}

#[test]
fn ignores_apps_without_proton_in_the_name() {
    let steam = FakeSteam::new();
    steam.proton_app(4_000, "Garry's Mod", "GarrysMod");
    steam.proton_app(2_805_730, "Proton 9.0", "Proton 9.0");

    let runners = find_all_versions_in(&steam.steam_dir(), NO_COMPAT_DIRS).unwrap();

    assert_eq!(names(&runners), BTreeSet::from(["Proton 9.0"]));
}

#[test]
fn skips_proton_app_without_a_proton_binary() {
    let steam = FakeSteam::new();
    steam.app(2_805_730, "Proton 9.0", "Proton 9.0");
    steam.proton_app(2_348_590, "Proton 8.0", "Proton 8.0");

    let runners = find_all_versions_in(&steam.steam_dir(), NO_COMPAT_DIRS).unwrap();

    assert_eq!(names(&runners), BTreeSet::from(["Proton 8.0"]));
}

#[test]
fn bails_when_no_versions_are_found() {
    let steam = FakeSteam::new();
    steam.proton_app(4_000, "Garry's Mod", "GarrysMod");

    let err = find_all_versions_in(&steam.steam_dir(), NO_COMPAT_DIRS).unwrap_err();

    assert!(err.to_string().contains("No Proton versions found"));
}

#[test]
fn finds_custom_runners_in_compat_dirs() {
    let steam = FakeSteam::new();
    let compat_dir = steam.steam_compat_dir();
    steam.compat_tool(&compat_dir, "GE-Proton9-27");

    let runners = find_custom_versions_in(&[compat_dir], &steam.steam_dir()).unwrap();

    assert_eq!(runners.len(), 1);
    assert_eq!(runners[0].pretty_name, "GE-Proton9-27");
    // Custom runners keep the directory name verbatim for both fields.
    assert_eq!(runners[0].name, "GE-Proton9-27");
    // compatibilitytools.d entries are custom; "Automatic" must never pick them.
    assert!(runners[0].is_custom);
}

#[test]
fn ignores_compat_dir_entries_without_a_proton_binary() {
    let steam = FakeSteam::new();
    let compat_dir = steam.steam_compat_dir();
    fs::create_dir_all(compat_dir.join("not-a-runner")).unwrap();
    fs::write(compat_dir.join("stray-file"), "").unwrap();
    steam.compat_tool(&compat_dir, "GE-Proton9-27");

    let runners = find_custom_versions_in(&[compat_dir], &steam.steam_dir()).unwrap();

    assert_eq!(names(&runners), BTreeSet::from(["GE-Proton9-27"]));
}

#[test]
fn collects_from_every_compat_dir() {
    let steam = FakeSteam::new();
    let steam_compat = steam.steam_compat_dir();
    let system_compat = steam.root.join("system-compat");
    fs::create_dir_all(&system_compat).unwrap();
    steam.compat_tool(&steam_compat, "GE-Proton9-27");
    steam.compat_tool(&system_compat, "proton-cachyos-native");

    let runners =
        find_custom_versions_in(&[steam_compat, system_compat], &steam.steam_dir()).unwrap();

    assert_eq!(
        names(&runners),
        BTreeSet::from(["GE-Proton9-27", "proton-cachyos-native"])
    );
}

#[test]
fn missing_compat_dirs_are_not_an_error() {
    let steam = FakeSteam::new();

    let runners =
        find_custom_versions_in(&[steam.root.join("does-not-exist")], &steam.steam_dir()).unwrap();

    assert!(runners.is_empty());
}

#[test]
fn combines_library_and_custom_runners() {
    let steam = FakeSteam::new();
    steam.proton_app(2_805_730, "Proton 9.0", "Proton 9.0");
    let compat_dir = steam.steam_compat_dir();
    steam.compat_tool(&compat_dir, "GE-Proton9-27");

    let runners = find_all_versions_in(&steam.steam_dir(), &[compat_dir]).unwrap();

    assert_eq!(
        names(&runners),
        BTreeSet::from(["Proton 9.0", "GE-Proton9-27"])
    );
}

#[test]
fn runner_resolves_an_installed_runtime() {
    let steam = FakeSteam::new();
    let proton_dir = steam.proton_app(2_805_730, "Proton 9.0", "Proton 9.0");
    steam.toolmanifest(&proton_dir, 1_628_350);
    let runtime_dir = steam.app(
        1_628_350,
        "Steam Linux Runtime 3.0 (sniper)",
        "SteamLinuxRuntime_sniper",
    );

    let runners = find_all_versions_in(&steam.steam_dir(), NO_COMPAT_DIRS).unwrap();

    assert_eq!(runners.len(), 1);
    let runtime = runners[0].runtime.as_ref().expect("runtime should resolve");
    assert_eq!(runtime.name, "Steam Linux Runtime 3.0 (sniper)");
    assert_eq!(runtime.pretty_name, "Steam Linux Runtime 3.0 (sniper)");
    assert_eq!(runtime.path, runtime_dir);
}

#[test]
fn runner_with_a_missing_runtime_is_skipped() {
    let steam = FakeSteam::new();
    let needs_runtime = steam.proton_app(2_805_730, "Proton 9.0", "Proton 9.0");
    steam.toolmanifest(&needs_runtime, 1_628_350);
    steam.proton_app(2_348_590, "Proton 8.0", "Proton 8.0");

    let runners = find_all_versions_in(&steam.steam_dir(), NO_COMPAT_DIRS).unwrap();

    // The runner requiring an uninstalled runtime is dropped; the other one survives.
    assert_eq!(names(&runners), BTreeSet::from(["Proton 8.0"]));
}

#[test]
fn custom_runner_with_a_missing_runtime_is_skipped() {
    let steam = FakeSteam::new();
    let compat_dir = steam.steam_compat_dir();
    let needs_runtime = steam.compat_tool(&compat_dir, "GE-Proton9-27");
    steam.toolmanifest(&needs_runtime, 1_628_350);
    steam.compat_tool(&compat_dir, "proton-cachyos-native");

    let runners = find_custom_versions_in(&[compat_dir], &steam.steam_dir()).unwrap();

    assert_eq!(names(&runners), BTreeSet::from(["proton-cachyos-native"]));
}

/// A runner as the Steam library scan would build it.
fn official(pretty_name: &str) -> Runner {
    Runner {
        name: pretty_name.to_lowercase().replace(' ', "_"),
        pretty_name: pretty_name.to_string(),
        path: PathBuf::from("/nonexistent/proton"),
        runtime: None,
        is_custom: false,
    }
}

/// A runner as the compatibilitytools.d scan would build it.
fn custom(pretty_name: &str) -> Runner {
    Runner {
        is_custom: true,
        ..official(pretty_name)
    }
}

#[test]
fn picks_the_highest_numbered_proton() {
    let versions = [
        official("Proton 8.0"),
        official("Proton 9.0-4"),
        official("Proton 7.0"),
    ];

    assert_eq!(
        find_highest_version(&versions).unwrap().pretty_name,
        "Proton 9.0-4"
    );
}

#[test]
fn a_numbered_proton_beats_experimental() {
    let versions = [official("Proton Experimental"), official("Proton 9.0")];

    assert_eq!(
        find_highest_version(&versions).unwrap().pretty_name,
        "Proton 9.0"
    );
}

#[test]
fn automatic_never_picks_custom_runners() {
    // Intentional: "Automatic (Recommended)" must only ever land on an official Valve
    // Proton. Custom runners are identified by the `is_custom` provenance flag (set at
    // collection time), not by their name, and always rank below every official.
    let versions = [
        custom("GE-Proton9-27"),
        official("Proton 8.0"),
        custom("proton-cachyos-native"),
    ];

    assert_eq!(
        find_highest_version(&versions).unwrap().pretty_name,
        "Proton 8.0"
    );
}

#[test]
fn experimental_outranks_hotfix() {
    // Both are real Steam apps; a machine can have them without any numbered Proton.
    assert_eq!(
        find_highest_version(&[official("Proton Hotfix"), official("Proton Experimental")])
            .unwrap()
            .pretty_name,
        "Proton Experimental"
    );
}

#[test]
fn falls_back_to_a_custom_runner_when_thats_all_there_is() {
    // Unreachable from the install menu (select_version hides "Automatic" when only custom
    // runners exist), but steam_game::get_runner still relies on this at launch time when
    // the configured runner has gone missing: launching with *some* runner beats erroring.
    let versions = [custom("GE-Proton9-27"), custom("proton-cachyos-native")];

    assert!(find_highest_version(&versions).is_some());
}
