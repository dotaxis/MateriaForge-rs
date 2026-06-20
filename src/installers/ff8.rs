use anyhow::{Context, Result};
use lib_game_detector::data::{Game as DetectedGame, SupportedLaunchers};

use crate::{
    gamelib_helper::{self, PrefixedGame},
    resource_handler,
};

use super::{
    common,
    game_installer::{Detection, GameInstaller},
    target::{InstallTarget, FF8_APPID},
};

pub struct FF8Installer;

pub const INSTALLER: FF8Installer = FF8Installer;

impl GameInstaller for FF8Installer {
    fn target(&self) -> InstallTarget {
        InstallTarget::FF8
    }

    fn menu_label(&self) -> &'static str {
        "Install Junction VIII for FF8"
    }

    fn detect(&self, installs: &[DetectedGame]) -> Detection {
        let steam_index = installs.iter().position(|game| {
            let title = game.title.to_lowercase();
            (title.contains("final fantasy viii") || title.contains("final fantasy 8"))
                && game.source == SupportedLaunchers::Steam
        });

        Detection {
            steam_index,
            alt_index: None,
        }
    }

    fn choose_detected_index(
        &self,
        _installs: &[DetectedGame],
        detection: Detection,
    ) -> Result<Option<usize>> {
        Ok(detection.steam_index)
    }

    fn steam_search_label(&self) -> &'static str {
        "FF8"
    }

    fn resolve_steam_game(
        &self,
        steam_dir: steamlocate::SteamDir,
    ) -> Result<gamelib_helper::steam_game::SteamGame> {
        gamelib_helper::steam_game::get_game(FF8_APPID, steam_dir)
            .context("Couldn't find Steam installation of FF8")
    }

    fn patch_install(
        &self,
        install_path: &std::path::Path,
        game: &dyn PrefixedGame,
        update_channel: &str,
    ) -> Result<()> {
        let ff8_user_base = common::wine_user_documents_dir(game.prefix())
            .join("Square Enix/FINAL FANTASY VIII Steam");
        common::ensure_wine_user_slot(&ff8_user_base, "user_12345678")?;

        let mut settings_xml = resource_handler::as_str(
            "settings.xml".to_string(),
            install_path.join("J8Workshop"),
            resource_handler::JUNCTION_VIII_SETTINGS_XML,
        );
        let full = game.path().join("FF8_en.exe").to_string_lossy().to_string();
        let trimmed = full
            .find("/steamapps/")
            .map_or(full.as_str(), |i| &full[i..]);
        let ff8_exe = format!("S:{}", trimmed.replace('/', "\\"));

        let library_location = format!(
            "Z:{}",
            install_path.join("mods").to_string_lossy().replace('/', "\\")
        );

        settings_xml.contents = settings_xml
            .contents
            .replace("LIBRARY_LOCATION", &library_location)
            .replace("FF8_EXE", &ff8_exe)
            .replace("UPDATE_CHANNEL", update_channel);
        settings_xml.write()?;

        Ok(())
    }
}
