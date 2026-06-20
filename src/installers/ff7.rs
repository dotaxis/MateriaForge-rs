use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use lib_game_detector::data::{Game as DetectedGame, SupportedLaunchers};

use crate::{
    gamelib_helper::{self, gog_game, PrefixedGame},
    resource_handler,
};

use super::{
    common,
    game_installer::{Detection, GameInstaller},
    target::{InstallTarget, FF7_2026_APPID, FF7_APPID, FF7_GOG_APPID},
};

pub struct FF7Installer;

pub const INSTALLER: FF7Installer = FF7Installer;

impl GameInstaller for FF7Installer {
    fn target(&self) -> InstallTarget {
        InstallTarget::FF7
    }

    fn menu_label(&self) -> &'static str {
        "Install 7th Heaven for FF7"
    }

    fn detect(&self, installs: &[DetectedGame]) -> Detection {
        let steam_index = installs.iter().position(|game| {
            game.title.to_lowercase().contains("final fantasy vii")
                && game.source == SupportedLaunchers::Steam
        });
        let alt_index = installs.iter().position(|game| {
            game.title.to_lowercase().contains("final fantasy vii")
                && game.source == SupportedLaunchers::HeroicGamesGOG
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
                .with_prompt("Multiple versions of FF7 detected. Which one do you want to use?")
                .default(0)
                .items(choices)
                .interact()?;

            match selection {
                0 => steam,
                1 => gog,
                _ => unreachable!(),
            }
        }
            (None, Some(gog)) => {
            log::info!("Heroic Games Launcher install detected!");
            gog
        }
            (Some(steam), None) => {
            log::info!("Steam install detected!");
            steam
        }
            (None, None) => return Ok(None),
        };

        Ok(Some(selected))
    }

    fn steam_search_label(&self) -> &'static str {
        "FF7"
    }

    fn resolve_steam_game(
        &self,
        steam_dir: steamlocate::SteamDir,
    ) -> Result<gamelib_helper::steam_game::SteamGame> {
        let original = gamelib_helper::steam_game::get_game(FF7_APPID, steam_dir.clone()).ok();
        let remaster =
            gamelib_helper::steam_game::get_game(FF7_2026_APPID, steam_dir.clone()).ok();

        if original.is_none() && remaster.is_none() {
            bail!("Couldn't find any supported Steam version of FF7");
        }

        match (original, remaster) {
            (Some(og), Some(rm)) => {
                let choices = &[&og.name, &format!("{} (2026)", rm.name)];
                let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(
                        "Multiple Steam installations of FF7 were detected. Which one do you want to patch?",
                    )
                    .default(0)
                    .items(choices)
                    .interact()
                    .context("Selection failed")?;

                match selection {
                    0 => Ok(og),
                    1 => Ok(rm),
                    _ => unreachable!(),
                }
            }
            (Some(og), None) => Ok(og),
            (None, Some(rm)) => Ok(rm),
            (None, None) => unreachable!(),
        }
    }

    fn resolve_nonsteam_game(
        &self,
        found_game: &DetectedGame,
    ) -> Result<(Box<dyn PrefixedGame>, &'static str)> {
        let game = Box::new(
            gog_game::get_game(FF7_GOG_APPID, found_game)
                .context("Failed to get GOG game details")?,
        );
        Ok((game, "gog"))
    }

    fn patch_install(
        &self,
        install_path: &std::path::Path,
        game: &dyn PrefixedGame,
        update_channel: &str,
    ) -> Result<()> {
        if game.app_id() == FF7_APPID {
            let ff7_user_base = common::wine_user_documents_dir(game.prefix())
                .join("Square Enix/FINAL FANTASY VII Steam");
            common::ensure_wine_user_slot(&ff7_user_base, "user_12345678")?;
        }

        let mut settings_xml = resource_handler::as_str(
            "settings.xml".to_string(),
            install_path.join("7thWorkshop"),
            resource_handler::SEVENTH_HEAVEN_SETTINGS_XML,
        );

        let ff7_version = match game.app_id() {
            FF7_APPID => "Steam",
            FF7_2026_APPID => "SteamReRelease",
            FF7_GOG_APPID => "GOG",
            _ => "Unknown",
        };
        let ff7_exe = match game.app_id() {
            FF7_APPID => "ff7_en.exe",
            FF7_2026_APPID => "FFVII.exe",
            FF7_GOG_APPID => "FFVII.exe",
            _ => "FFVII.exe",
        };
        let ff7_exe_path = match game.app_id() {
            FF7_GOG_APPID => format!(
                "Z:{}",
                game.path().join(ff7_exe).to_string_lossy().replace('/', "\\")
            ),
            _ => {
                let full = game.path().join(ff7_exe).to_string_lossy().to_string();
                let trimmed = full
                    .find("/steamapps/")
                    .map_or(full.as_str(), |i| &full[i..]);
                format!("S:{}", trimmed.replace('/', "\\"))
            }
        };

        let library_location = format!(
            "Z:{}",
            install_path.join("mods").to_string_lossy().replace('/', "\\")
        );

        settings_xml.contents = settings_xml
            .contents
            .replace("LIBRARY_LOCATION", &library_location)
            .replace("FF7_EXE", &ff7_exe_path)
            .replace("FF7_VERSION", ff7_version)
            .replace("UPDATE_CHANNEL", update_channel);

        settings_xml.write()
    }
}
