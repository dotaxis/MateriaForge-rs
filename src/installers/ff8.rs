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

pub(super) const FF8_APPID: u32 = 39150;
pub(super) const FF8_REMASTERED_APPID: u32 = 1026680;
pub(super) const FF8_GOG_APPID: u32 = 1086370078;

pub struct FF8Target;
pub static TARGET: FF8Target = FF8Target;

impl InstallTarget for FF8Target {
    fn target_key(&self) -> &'static str {
        "ff8"
    }

    fn mod_loader_name(&self) -> &'static str {
        "Junction VIII"
    }

    fn mod_loader_repo(&self) -> &'static str {
        "tsunamods-codes/Junction-VIII"
    }

    fn install_log_name(&self) -> &'static str {
        "JunctionVIII.log"
    }

    fn launch_binary_name(&self) -> &'static str {
        "Launch Junction VIII"
    }

    fn desktop_template(&self) -> &'static str {
        resource_handler::JUNCTION_VIII_SHORTCUT_FILE
    }

    fn desktop_file_name(&self, _app_id: u32) -> String {
        "Junction VIII.desktop".to_string()
    }

    fn icon_name(&self) -> &'static str {
        "Junction-VIII"
    }

    fn icon_file_name(&self) -> &'static str {
        "Junction-VIII.png"
    }

    fn icon_bytes(&self) -> &'static [u8] {
        resource_handler::JUNCTION_LOGO_PNG
    }
}

pub struct FF8Installer;

pub const INSTALLER: FF8Installer = FF8Installer;

fn is_ff8_steam_app_id(app_id: u32) -> bool {
    matches!(app_id, FF8_APPID | FF8_REMASTERED_APPID)
}

fn is_ff8_gog_app_id(app_id: u32) -> bool {
    app_id == FF8_GOG_APPID
}

fn ensure_ff8_user_slot(game: &dyn PrefixedGame) -> Result<()> {
    match game.app_id() {
        FF8_APPID => {
            let ff8_user_base = common::wine_user_documents_dir(game.prefix())
                .join("Square Enix/FINAL FANTASY VIII Steam");
            common::ensure_wine_user_slot(&ff8_user_base, "user_12345678")?;
        }
        FF8_REMASTERED_APPID | FF8_GOG_APPID => {
            let edition = match game.app_id() {
                FF8_GOG_APPID => "GOG",
                _ => "Steam",
            };
            let ff8_user_base = common::wine_user_documents_dir(game.prefix())
                .join("My Games")
                .join("FINAL FANTASY VIII Remastered")
                .join(edition);
            common::ensure_wine_user_slot(&ff8_user_base, "0/game_data/user/saves")?;
        }
        _ => {}
    }

    Ok(())
}

impl GameInstaller for FF8Installer {
    fn target(&self) -> &'static dyn InstallTarget {
        &TARGET
    }

    fn menu_label(&self) -> &'static str {
        "Install Junction VIII for FF8"
    }

    fn detect(&self, installs: &[DetectedGame]) -> Detection {
        let steam_index = installs.iter().position(|game| {
            game.source == SupportedLaunchers::Steam
                && detected_app_id(game).is_some_and(is_ff8_steam_app_id)
        });
        let alt_index = installs.iter().position(|game| {
            game.source == SupportedLaunchers::HeroicGamesGOG
                && detected_app_id(game).is_some_and(is_ff8_gog_app_id)
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
                    .with_prompt("Multiple versions of FF8 detected. Which one do you want to use?")
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
        let original = gamelib_helper::steam_game::get_game(FF8_APPID, steam_dir.clone()).ok();
        let remaster = gamelib_helper::steam_game::get_game(FF8_REMASTERED_APPID, steam_dir).ok();

        if original.is_none() && remaster.is_none() {
            bail!("Couldn't find any supported Steam version of FF8");
        }

        match (original, remaster) {
            (Some(og), Some(rm)) => {
                let choices = &[&og.name, &format!("{} (Remastered)", rm.name)];
                let default_selection = match preferred_app_id {
                    Some(FF8_REMASTERED_APPID) => 1,
                    _ => 0,
                };
                let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(
                        "Multiple Steam installations of FF8 were detected. Which one do you want to patch?",
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
            gog_game::get_game(FF8_GOG_APPID, found_game)
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
        ensure_ff8_user_slot(game)?;

        let mut settings_xml = resource_handler::as_str(
            "settings.xml".to_string(),
            install_path.join("J8Workshop"),
            resource_handler::JUNCTION_VIII_SETTINGS_XML,
        );
        let ff8_version = match game.app_id() {
            FF8_APPID => "Steam",
            FF8_REMASTERED_APPID => "Remastered",
            FF8_GOG_APPID => "GOG",
            _ => "Unknown",
        };
        let ff8_exe_name = match game.app_id() {
            FF8_APPID => "FF8_en.exe",
            FF8_REMASTERED_APPID | FF8_GOG_APPID => "ff8.exe",
            _ => "ff8.exe",
        };
        let ff8_exe = match game.app_id() {
            FF8_GOG_APPID => format!(
                "Z:{}",
                game.path()
                    .join(ff8_exe_name)
                    .to_string_lossy()
                    .replace('/', "\\")
            ),
            _ => {
                let full = game.path().join(ff8_exe_name).to_string_lossy().to_string();
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
            .replace("FF8_EXE", &ff8_exe)
            .replace("FF8_VERSION", ff8_version)
            .replace("UPDATE_CHANNEL", update_channel);
        settings_xml.write()?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/ff8.rs"]
mod tests;
