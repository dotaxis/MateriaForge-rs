use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use lib_game_detector::data::{Game as DetectedGame, SupportedLaunchers};

use crate::{
    gamelib_helper::{self, gog_game, PrefixedGame},
    resource_handler,
};

use super::{
    common, detected_app_id,
    game_installer::{Detection, GameInstaller},
    target::InstallTarget,
};

pub(super) const FF7_APPID: u32 = 39140;
pub(super) const FF7_2026_APPID: u32 = 3837340;
pub(super) const FF7_GOG_APPID: u32 = 1698970154;

pub struct FF7Target;
pub static TARGET: FF7Target = FF7Target;

impl InstallTarget for FF7Target {
    fn target_key(&self) -> &'static str {
        "ff7"
    }

    fn mod_loader_name(&self) -> &'static str {
        "7th Heaven"
    }

    fn mod_loader_repo(&self) -> &'static str {
        "tsunamods-codes/7th-Heaven"
    }

    fn install_log_name(&self) -> &'static str {
        "7thHeaven.log"
    }

    fn launch_binary_name(&self) -> &'static str {
        "Launch 7th Heaven"
    }

    fn desktop_template(&self) -> &'static str {
        resource_handler::SEVENTH_HEAVEN_SHORTCUT_FILE
    }

    fn desktop_file_name(&self, app_id: u32) -> String {
        format!("7th Heaven {}.desktop", self.shortcut_identifier(app_id))
    }

    fn icon_name(&self) -> &'static str {
        "7th-heaven"
    }

    fn icon_file_name(&self) -> &'static str {
        "7th-heaven.png"
    }

    fn icon_bytes(&self) -> &'static [u8] {
        resource_handler::LOGO_PNG
    }

    fn shortcut_identifier(&self, app_id: u32) -> &'static str {
        match app_id {
            FF7_APPID => "(2013)",
            FF7_2026_APPID => "(2026)",
            FF7_GOG_APPID => "(GOG)",
            _ => "",
        }
    }
}

pub struct FF7Installer;

pub const INSTALLER: FF7Installer = FF7Installer;

fn is_ff7_steam_app_id(app_id: u32) -> bool {
    matches!(app_id, FF7_APPID | FF7_2026_APPID)
}

fn is_ff7_gog_app_id(app_id: u32) -> bool {
    app_id == FF7_GOG_APPID
}

impl GameInstaller for FF7Installer {
    fn target(&self) -> &'static dyn InstallTarget {
        &TARGET
    }

    fn menu_label(&self) -> &'static str {
        "Install 7th Heaven for FF7"
    }

    fn detect(&self, installs: &[DetectedGame]) -> Detection {
        let steam_index = installs.iter().position(|game| {
            game.source == SupportedLaunchers::Steam
                && detected_app_id(game).is_some_and(is_ff7_steam_app_id)
        });
        let alt_index = installs.iter().position(|game| {
            game.source == SupportedLaunchers::HeroicGamesGOG
                && detected_app_id(game).is_some_and(is_ff7_gog_app_id)
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

    fn resolve_steam_game(
        &self,
        steam_dir: steamlocate::SteamDir,
        preferred_app_id: Option<u32>,
    ) -> Result<gamelib_helper::steam_game::SteamGame> {
        let original = gamelib_helper::steam_game::get_game(FF7_APPID, steam_dir.clone()).ok();
        let remaster = gamelib_helper::steam_game::get_game(FF7_2026_APPID, steam_dir.clone()).ok();

        if original.is_none() && remaster.is_none() {
            bail!("Couldn't find any supported Steam version of FF7");
        }

        match (original, remaster) {
            (Some(og), Some(rm)) => {
                let choices = &[&og.name, &format!("{} (2026)", rm.name)];
                let default_selection = match preferred_app_id {
                    Some(FF7_2026_APPID) => 1,
                    _ => 0,
                };
                let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(
                        "Multiple Steam installations of FF7 were detected. Which one do you want to patch?",
                    )
                    .default(default_selection)
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
                game.path()
                    .join(ff7_exe)
                    .to_string_lossy()
                    .replace('/', "\\")
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
            install_path
                .join("mods")
                .to_string_lossy()
                .replace('/', "\\")
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

#[cfg(test)]
#[path = "../tests/ff7.rs"]
mod tests;
