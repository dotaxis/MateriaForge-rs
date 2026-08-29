use anyhow::{bail, Result};
use lib_game_detector::data::Game as DetectedGame;
use std::path::{Path, PathBuf};

use crate::gamelib_helper::{steam_game::SteamGame, PrefixedGame};

use super::{common, target::InstallTarget};

#[derive(Clone, Copy, Debug, Default)]
pub struct Detection {
    pub steam_index: Option<usize>,
    pub alt_index: Option<usize>,
}

impl Detection {
    pub fn is_detected(self) -> bool {
        self.steam_index.is_some() || self.alt_index.is_some()
    }
}

pub trait GameInstaller {
    fn target(&self) -> &'static dyn InstallTarget;
    fn menu_label(&self) -> &'static str;
    fn detect(&self, installs: &[DetectedGame]) -> Detection;
    fn choose_detected_index(
        &self,
        installs: &[DetectedGame],
        detection: Detection,
    ) -> Result<Option<usize>>;

    fn resolve_steam_game(
        &self,
        steam_dir: steamlocate::SteamDir,
        preferred_app_id: Option<u32>,
    ) -> Result<SteamGame>;

    fn resolve_nonsteam_game(
        &self,
        _found_game: &DetectedGame,
    ) -> Result<(Box<dyn PrefixedGame>, &'static str)> {
        bail!(
            "Non-Steam launcher is not supported for {}",
            self.target().mod_loader_name()
        )
    }

    fn patch_install(
        &self,
        install_path: &std::path::Path,
        game: &dyn PrefixedGame,
        update_channel: &str,
    ) -> Result<()>;

    fn requires_runner_selection(&self) -> bool {
        true
    }

    fn requires_install_path(&self) -> bool {
        true
    }

    fn should_write_launcher_config(&self) -> bool {
        true
    }

    fn should_run_patch_install(&self) -> bool {
        true
    }

    fn should_manage_shortcuts(&self) -> bool {
        true
    }

    fn needs_common_patch_files(&self) -> bool {
        true
    }

    fn run_loader_install(
        &self,
        game: &dyn PrefixedGame,
        installer_path: PathBuf,
        install_path: &Path,
    ) -> Result<()> {
        common::install_loader(self.target(), game, installer_path, install_path)
    }
}
