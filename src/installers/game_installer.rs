use anyhow::{bail, Context, Result};
use lib_game_detector::data::Game as DetectedGame;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    config_handler,
    gamelib_helper::{steam_game::SteamGame, PrefixedGame, DEFAULT_WINEDEBUG},
};

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
        _install_path: &std::path::Path,
        _game: &dyn PrefixedGame,
        _update_channel: &str,
    ) -> Result<()> {
        Ok(())
    }

    fn requires_runner_selection(&self) -> bool {
        true
    }

    fn pre_install(&self, config: HashMap<&str, String>) -> Result<()> {
        let mut env_vars = HashMap::new();
        env_vars.insert("WINEDEBUG", DEFAULT_WINEDEBUG.to_string());
        config_handler::write(config, env_vars).context("Failed to write config")
    }

    fn install(&self, game: &dyn PrefixedGame, installer_path: PathBuf) -> Result<PathBuf> {
        let install_path = common::get_install_path(self.target())?;
        common::install_loader(self.target(), game, installer_path, &install_path)?;
        Ok(install_path)
    }

    fn post_install(
        &self,
        install_path: &Path,
        game: &dyn PrefixedGame,
        steam_dir: Option<steamlocate::SteamDir>,
        is_deck: bool,
        update_channel: &str,
    ) -> Result<()> {
        common::write_common_patch_files(install_path, game)?;
        self.patch_install(install_path, game, update_channel)?;
        let steam_shortcut = common::create_shortcuts(
            self.target(),
            install_path,
            steam_dir.clone(),
            game.app_id(),
        )
        .context("Failed to create shortcuts")?;
        common::add_controller_config(game, &steam_dir, steam_shortcut, is_deck)
            .context("Failed to set controller config")
    }
}
