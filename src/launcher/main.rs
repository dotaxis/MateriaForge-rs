use anyhow::{bail, Context, Result};
use materia_forge::{config_handler, logging, steam_helper};
use std::{env, path::Path};

static FF7_APPID: u32 = 39140;
static FF7_2026_APPID: u32 = 3837340;

fn main() -> Result<()> {
    logging::init()?;

    let launcher_bin = env::current_exe().expect("Failed to get binary path");
    let launcher_dir = launcher_bin
        .parent()
        .expect("Failed to get binary directory");
    let seventh_heaven_exe = launcher_dir.join("7th Heaven.exe");

    if !seventh_heaven_exe.exists() {
        bail!("Couldn't find '7th Heaven.exe'!");
    }

    let steam_dir_str = &config_handler::read_value("steam_dir")?;
    let steam_dir = steamlocate::SteamDir::from_dir(Path::new(steam_dir_str))?;
    log::info!("Steam path: {}", steam_dir.path().display());

    let game = steam_helper::game::get_game(FF7_APPID, steam_dir.clone())
    .or_else(|_| steam_helper::game::get_game(FF7_2026_APPID, steam_dir.clone()))
    .with_context(|| "Couldn't find either FF7 or FF7 2026 Edition in the Steam library.")?;

     if let Some(runner) = &game.runner {
        log::info!("Found runner: {}", runner.name);
    } else {
        log::info!("No runner found for the game.");
    }

    steam_helper::game::launch_exe_in_prefix(seventh_heaven_exe, &game, None)
        .context("Failed to launch 7th Heaven.")?;

    Ok(())
}
