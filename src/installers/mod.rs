use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use lib_game_detector::get_detector;

use crate::{
    config_handler,
    gamelib_helper::{self, PrefixedGame, DEFAULT_WINEDEBUG},
};

mod common;
mod ff7;
mod ff8;
mod game_installer;
mod target;

use game_installer::{Detection, GameInstaller};

pub(super) fn detected_app_id(game: &lib_game_detector::data::Game) -> Option<u32> {
    game.launch_command
        .get_args()
        .filter_map(|arg| arg.to_str())
        .find_map(|arg| {
            arg.strip_prefix("steam://rungameid/")
                .or_else(|| arg.split("steam://rungameid/").nth(1))
                .or_else(|| arg.strip_prefix("heroic://launch/gog/")
                    .or_else(|| arg.split("heroic://launch/gog/").nth(1)))
                .and_then(|value| value.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok())
        })
}

pub fn run(is_deck: bool) -> Result<()> {
    let detector = get_detector();
    let installs = detector.get_all_detected_games();
    let installers = installers_registry();

    let mut available: Vec<(&dyn GameInstaller, Detection)> = installers
        .iter()
        .map(|installer| (*installer, installer.detect(&installs)))
        .filter(|(_, detection)| detection.is_detected())
        .collect();

    if available.is_empty() {
        bail!("Couldn't find any supported game installations!");
    }

    if available.len() > 1 {
        let choices: Vec<&str> = available
            .iter()
            .map(|(installer, _)| installer.menu_label())
            .collect();
        let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "Multiple supported games were detected. Which installer do you want to run?",
            )
            .default(0)
            .items(&choices)
            .interact()?;

        let (installer, detection) = available.remove(selection);
        let game_index = installer
            .choose_detected_index(&installs, detection)?
            .with_context(|| {
                format!(
                    "{} was selected, but no matching installation was found",
                    installer.menu_label()
                )
            })?;

        return run_install(installer, &installs[game_index], is_deck);
    }

    let (installer, detection) = available.remove(0);
    let game_index = installer
        .choose_detected_index(&installs, detection)?
        .with_context(|| {
            format!(
                "{} was detected, but no matching installation was found",
                installer.menu_label()
            )
        })?;

    run_install(installer, &installs[game_index], is_deck)
}

fn run_install(
    installer: &dyn GameInstaller,
    found_game: &lib_game_detector::data::Game,
    is_deck: bool,
) -> Result<()> {
    let target = installer.target();

    let mut config = std::collections::HashMap::new();
    config.insert("target", target.target_key().to_string());

    let game: Box<dyn PrefixedGame>;
    let steam_dir: Option<steamlocate::SteamDir> = gamelib_helper::steam_lib::get_library().ok();

    if found_game.source == lib_game_detector::data::SupportedLaunchers::Steam {
        if steam_dir.is_none() {
            bail!("Selected Steam game, but no Steam library could be found?");
        }
        config.insert("type", "steam".to_string());

        let steam_dir = gamelib_helper::steam_lib::get_library()?;
        config.insert("steam_dir", steam_dir.path().display().to_string());

        let find_message = format!("Finding {}...", installer.steam_search_label());
        let mut steam_game = common::with_spinner(&find_message, "Done!", || {
            installer.resolve_steam_game(steam_dir.clone(), detected_app_id(found_game))
        })?;

        steam_game.runner = Some(gamelib_helper::steam_game::select_runner(&steam_game)?);
        game = Box::new(steam_game);
    } else {
        let (resolved_game, install_type) = installer.resolve_nonsteam_game(found_game)?;
        config.insert("type", install_type.to_string());
        game = resolved_game;
    }

    config.insert("app_id", game.app_id().to_string());

    let choices = &["Yes", "No"];
    let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you want to continue installing {}?",
            target.mod_loader_name()
        ))
        .default(0)
        .items(choices)
        .interact()?;

    if selection == 1 {
        println!("Understood. Exiting.");
        std::process::exit(0);
    }

    let use_canary = std::env::args().any(|a| a == "-c" || a == "--canary");
    let update_channel = if use_canary { "Canary" } else { "Stable" };

    let cache_dir = home::home_dir().context("Couldn't find $HOME?")?.join(".cache");
    let exe_path = common::download_asset(target.mod_loader_repo(), cache_dir, use_canary)
        .with_context(|| format!("Failed to download {}", target.mod_loader_name()))?;

    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("WINEDEBUG", DEFAULT_WINEDEBUG.to_string());
    config_handler::write(config, env_vars).context("Failed to write config")?;

    let install_path = common::get_install_path(target)?;
    common::with_spinner(
        &format!("Installing {}...", target.mod_loader_name()),
        "Done!",
        || common::install_loader(target, game.as_ref(), exe_path, &install_path),
    )?;

    common::with_spinner("Patching installation...", "Done!", || {
        patch_install(installer, &install_path, game.as_ref(), update_channel)
    })?;

    let steam_shortcut = common::create_shortcuts(target, &install_path, steam_dir.clone(), game.app_id())
        .context("Failed to create shortcuts")?;

    common::add_controller_config(game.as_ref(), &steam_dir, steam_shortcut, is_deck)
        .context("Failed to set controller config")?;

    println!(
        "{} {} successfully installed to '{}'",
        console::style("✔").green(),
        target.mod_loader_name(),
        console::style(install_path.display()).bold().underlined().white()
    );

    Ok(())
}

fn patch_install(
    installer: &dyn GameInstaller,
    install_path: &std::path::Path,
    game: &dyn PrefixedGame,
    update_channel: &str,
) -> Result<()> {
    common::write_common_patch_files(install_path, game)?;
    installer.patch_install(install_path, game, update_channel)
}

fn installers_registry() -> Vec<&'static dyn GameInstaller> {
    vec![&ff7::INSTALLER, &ff8::INSTALLER]
}
