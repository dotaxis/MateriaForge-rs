pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use anyhow::{bail, Context, Result};
use lib_game_detector::data::SupportedLaunchers;
use std::{env, path::Path};

use materia_forge::gamelib_helper::{Game, PrefixRunner};
use materia_forge::{config_handler, gamelib_helper, logging};

static FF7_GOG_APPID: u32 = 1698970154;
static FF8_APPID: u32 = 39150;

fn run_exe<G: Game + PrefixRunner>(game: &G, exe: std::path::PathBuf) -> Result<()> {
    if let Some(runner) = game.runner() {
        log::info!("Found runner: {}", runner.name);
    } else {
        log::info!("No runner found for game");
    }

    let cli_args: Vec<String> = env::args().skip(1).collect();
    let args = if cli_args.is_empty() {
        config_handler::read_value("launch_args")
            .unwrap_or_else(|_| "".to_string())
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    } else {
        cli_args
    };

    if args.is_empty() {
        log::info!("No launch arguments provided");
    } else {
        log::info!("Launch arguments: {:?}", args);
    }

    game.run_in_prefix(exe, Some(args))
        .context("Failed to launch mod loader")?;
    Ok(())
}

fn main() -> Result<()> {
     if let Err(e) = logging::init("launcher.log") {
        eprintln!("Failed to create log file! Error: {e}");
        std::process::exit(1);
    }
    log::info!("Starting MateriaForge version {}", VERSION);

    let install_type = config_handler::read_value("type")
        .unwrap_or_else(|_| "steam".to_string())
        .to_lowercase();
    let app_id = config_handler::read_value("app_id")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    let install_target = config_handler::read_value("target")
        .unwrap_or_else(|_| {
            if app_id == Some(FF8_APPID) {
                "ff8".to_string()
            } else {
                "ff7".to_string()
            }
        })
        .to_lowercase();

    let launcher_bin = env::current_exe().context("Failed to get binary path")?;
    let launcher_dir = launcher_bin
        .parent()
        .context("Failed to get binary directory")?;
    let loader_exe = if install_target == "ff8" {
        launcher_dir.join("Junction VIII.exe")
    } else {
        launcher_dir.join("7th Heaven.exe")
    };

    if !loader_exe.exists() {
        bail!(
            "Couldn't find loader executable at '{}'",
            loader_exe.display()
        );
    }

    match install_type.as_str() {
        "gog" => {
            let g = lib_game_detector::get_detector()
                .get_all_detected_games_from_specific_launcher(SupportedLaunchers::HeroicGamesGOG);
            let heroic_game = g
                .iter()
                .flatten()
                .find(|game| game.title.to_lowercase().contains("final fantasy vii"))
                .unwrap();
            let game = gamelib_helper::gog_game::get_game(FF7_GOG_APPID, &heroic_game)
                .context("Configured type=gog, but GOG game was not found")?;
            run_exe(&game, loader_exe)?;
        }
        _ => {
            let steam_dir_str = config_handler::read_value("steam_dir")
                .context("Configured type=steam, but steam_dir is missing in TOML")?;
            let steam_dir = steamlocate::SteamDir::from_dir(Path::new(&steam_dir_str))?;
            log::info!("Steam path: {}", steam_dir.path().display());

            let app_id = config_handler::read_value("app_id")
                .context("Configured type=steam, but app_id is missing in TOML")?;
            log::info!("App ID: {}", app_id);

            let mut game = gamelib_helper::steam_game::get_game(app_id.parse()?, steam_dir.clone())
                .context(format!("Couldn't find {} in Steam library", app_id))?;
            game.runner = Some(gamelib_helper::steam_game::get_runner(&game)?);
            run_exe(&game, loader_exe)?;
        }
    }

    Ok(())
}
