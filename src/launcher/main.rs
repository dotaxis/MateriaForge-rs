use anyhow::{bail, Context, Result};
use lib_game_detector::data::SupportedLaunchers;
use std::{env, path::Path};

use materia_forge::{config_handler, gamelib_helper, logging};
use materia_forge::gamelib_helper::{Game, PrefixRunner};

static FF7_GOG_APPID: u32 = 1698970154;

fn run_exe<G: Game + PrefixRunner>(game: &G, exe: std::path::PathBuf) -> Result<()> {
    if let Some(runner) = game.runner() {
        log::info!("Found runner: {}", runner.name);
    } else {
        log::info!("No runner found for game");
    }

    game.run_in_prefix(exe, None)
        .context("Failed to launch 7th Heaven")?;
    Ok(())
}

fn main() -> Result<()> {
    logging::init()?;

    let launcher_bin = env::current_exe().context("Failed to get binary path")?;
    let launcher_dir = launcher_bin.parent().context("Failed to get binary directory")?;
    let seventh_heaven_exe = launcher_dir.join("7th Heaven.exe");

    if !seventh_heaven_exe.exists() {
        bail!("Couldn't find '7th Heaven.exe'!");
    }

    let install_type = config_handler::read_value("type")
        .unwrap_or_else(|_| "steam".to_string())
        .to_lowercase();

    match install_type.as_str() {
        "gog" => {
            let g = lib_game_detector::get_detector()
                .get_all_detected_games_from_specific_launcher(SupportedLaunchers::HeroicGamesGOG);
            let heroic_game = g.iter().flatten().find(|game| {
                game.title.to_lowercase().contains("final fantasy vii")
            }).unwrap();
            let game = gamelib_helper::gog_game::get_game(FF7_GOG_APPID, &heroic_game)
                .context("Configured type=gog, but GOG game was not found")?;
            run_exe(&game, seventh_heaven_exe)?;
        }
        _ => {
            let steam_dir_str = config_handler::read_value("steam_dir")
                .context("Configured type=steam, but steam_dir is missing in TOML")?;
            let steam_dir = steamlocate::SteamDir::from_dir(Path::new(&steam_dir_str))?;
            log::info!("Steam path: {}", steam_dir.path().display());

            let app_id = config_handler::read_value("app_id")
                .context("Configured type=steam, but app_id is missing in TOML")?;
            log::info!("App ID: {}", app_id);

            let game = gamelib_helper::steam_game::get_game(app_id.parse()?, steam_dir.clone())
                .context(format!("Couldn't find {} in Steam library", app_id))?;
            run_exe(&game, seventh_heaven_exe)?;
        }
    }

    Ok(())
}