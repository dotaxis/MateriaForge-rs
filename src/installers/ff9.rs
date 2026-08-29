use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use lib_game_detector::data::{Game as DetectedGame, SupportedLaunchers};
use std::{
    collections::HashMap, fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command,
};

use crate::gamelib_helper::{self, gog_game, PrefixedGame};

use super::{
    detected_app_id,
    game_installer::{Detection, GameInstaller},
    target::InstallTarget,
};

pub(super) const FF9_STEAM_APPID: u32 = 377840;
pub(super) const FF9_GOG_APPID: u32 = 1375008492;

pub struct FF9Target;
pub static TARGET: FF9Target = FF9Target;

impl InstallTarget for FF9Target {
    fn target_key(&self) -> &'static str {
        "ff9"
    }

    fn mod_loader_name(&self) -> &'static str {
        "Memoria"
    }

    fn mod_loader_repo(&self) -> &'static str {
        "Albeoris/Memoria"
    }

    fn install_log_name(&self) -> &'static str {
        "Memoria.log"
    }

    fn github_asset_pattern(&self) -> &'static str {
        "Memoria.Patcher-linux-x64"
    }
}

pub struct FF9Installer;

pub const INSTALLER: FF9Installer = FF9Installer;

fn is_ff9_steam_app_id(app_id: u32) -> bool {
    app_id == FF9_STEAM_APPID
}

fn is_ff9_gog_app_id(app_id: u32) -> bool {
    app_id == FF9_GOG_APPID
}

impl GameInstaller for FF9Installer {
    fn target(&self) -> &'static dyn InstallTarget {
        &TARGET
    }

    fn menu_label(&self) -> &'static str {
        "Install Memoria for FF9"
    }

    fn detect(&self, installs: &[DetectedGame]) -> Detection {
        let steam_index = installs.iter().position(|game| {
            game.source == SupportedLaunchers::Steam
                && detected_app_id(game).is_some_and(is_ff9_steam_app_id)
        });
        let alt_index = installs.iter().position(|game| {
            game.source == SupportedLaunchers::HeroicGamesGOG
                && detected_app_id(game).is_some_and(is_ff9_gog_app_id)
        });

        Detection {
            steam_index,
            alt_index,
        }
    }

    fn choose_detected_index(
        &self,
        _installs: &[DetectedGame],
        detection: Detection,
    ) -> Result<Option<usize>> {
        let selected = match (detection.steam_index, detection.alt_index) {
            (Some(steam), Some(gog)) => {
                let choices = &["Steam", "Heroic Games"];
                let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Multiple versions of FF9 detected. Which one do you want to use?")
                    .default(0)
                    .items(choices)
                    .interact()?;

                match selection {
                    0 => steam,
                    1 => gog,
                    _ => unreachable!(),
                }
            }
            (None, Some(gog)) => gog,
            (Some(steam), None) => steam,
            (None, None) => return Ok(None),
        };

        Ok(Some(selected))
    }

    fn resolve_steam_game(
        &self,
        steam_dir: steamlocate::SteamDir,
        _preferred_app_id: Option<u32>,
    ) -> Result<gamelib_helper::steam_game::SteamGame> {
        gamelib_helper::steam_game::get_game(FF9_STEAM_APPID, steam_dir)
            .context("Couldn't find Steam installation of FF9")
    }

    fn resolve_nonsteam_game(
        &self,
        found_game: &DetectedGame,
    ) -> Result<(Box<dyn PrefixedGame>, &'static str)> {
        let game = Box::new(
            gog_game::get_game(FF9_GOG_APPID, found_game)
                .context("Failed to get GOG game details")?,
        );
        Ok((game, "gog"))
    }

    fn pre_install(&self, _config: HashMap<&str, String>) -> Result<()> {
        Ok(())
    }

    fn requires_runner_selection(&self) -> bool {
        false
    }

    fn install(&self, game: &dyn PrefixedGame, installer_path: PathBuf) -> Result<PathBuf> {
        let installer_path = installer_path
            .canonicalize()
            .with_context(|| format!("Memoria patcher not found at {:?}", installer_path))?;

        let metadata = fs::metadata(&installer_path)
            .with_context(|| format!("Failed to read metadata for {}", installer_path.display()))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&installer_path, permissions).with_context(|| {
            format!(
                "Failed to mark Memoria patcher as executable: {}",
                installer_path.display()
            )
        })?;

        let status = Command::new(&installer_path)
            .arg(game.path())
            .status()
            .with_context(|| {
                format!(
                    "Failed to run Memoria patcher at {}",
                    installer_path.display()
                )
            })?;

        if !status.success() {
            bail!("Memoria patcher exited with an error: {status}");
        }

        Ok(game.path().to_path_buf())
    }

    fn post_install(
        &self,
        _install_path: &std::path::Path,
        _game: &dyn PrefixedGame,
        _steam_dir: Option<steamlocate::SteamDir>,
        _is_deck: bool,
        _update_channel: &str,
    ) -> Result<()> {
        Ok(())
    }
}
