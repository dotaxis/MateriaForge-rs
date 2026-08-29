use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use lib_game_detector::get_detector;

use crate::gamelib_helper::{self, PrefixedGame};

mod common;
mod ff7;
mod ff8;
mod ff9;
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
                .or_else(|| {
                    arg.strip_prefix("heroic://launch/gog/")
                        .or_else(|| arg.split("heroic://launch/gog/").nth(1))
                })
                .and_then(|value| {
                    value
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .ok()
                })
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
        let Some(steam_install_dir) = steam_dir.clone() else {
            bail!("Selected Steam game, but no Steam library could be found?");
        };
        config.insert("type", "steam".to_string());
        config.insert("steam_dir", steam_install_dir.path().display().to_string());

        let mut steam_game =
            installer.resolve_steam_game(steam_install_dir.clone(), detected_app_id(found_game))?;

        if installer.requires_runner_selection() {
            steam_game.runner = Some(gamelib_helper::steam_game::select_runner(&steam_game)?);
        }
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

    let cache_dir = home::home_dir()
        .context("Couldn't find $HOME?")?
        .join(".cache");
    let exe_path = common::download_asset(
        target.mod_loader_repo(),
        cache_dir,
        use_canary,
        target.github_asset_pattern(),
    )
    .with_context(|| format!("Failed to download {}", target.mod_loader_name()))?;

    installer.pre_install(config)?;

    let install_path = common::with_spinner(
        &format!("Installing {}...", target.mod_loader_name()),
        "Done!",
        || installer.install(game.as_ref(), exe_path),
    )?;

    common::with_spinner("Finalizing installation...", "Done!", || {
        installer.post_install(
            &install_path,
            game.as_ref(),
            steam_dir.clone(),
            is_deck,
            update_channel,
        )
    })?;

    println!(
        "{} {} successfully installed to '{}'",
        console::style("✔").green(),
        target.mod_loader_name(),
        console::style(install_path.display())
            .bold()
            .underlined()
            .white()
    );

    Ok(())
}

fn installers_registry() -> Vec<&'static dyn GameInstaller> {
    vec![&ff7::INSTALLER, &ff8::INSTALLER, &ff9::INSTALLER]
}

#[cfg(test)]
#[path = "../tests/installers.rs"]
mod tests;
