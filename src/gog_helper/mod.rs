use std::path::PathBuf;
use super::steam_helper::proton::Runner;
use anyhow::Context;
use dialoguer::Result;
use serde_json::Value;

pub struct GogGame {
    pub app_id: u32,
    pub name: String,
    pub path: PathBuf,
    pub prefix: PathBuf,
    pub runner: Option<Runner>,
}

pub fn get_game(app_id: u32) -> Result<GogGame> {
    let heroic_config_path = get_heroic_config_path();
    let game_json_path = heroic_config_path.join("GamesConfig").join("{}.json}").with_file_name(app_id.to_string() + ".json");
    let game_json_string = std::fs::read_to_string(game_json_path).expect("Failed to read GOG game JSON");

    let json: Value = serde_json::from_str(&game_json_string).expect("Failed to parse GOG game JSON");
    Ok(GogGame {
        app_id,
        name: "FF7".to_string(),
        path: PathBuf::new(),
        prefix: PathBuf::new(),
        runner: None,
    })
}

/// Get path to the Heroic Games Launcher config dir, falling back to the flatpak version if necessary
fn get_heroic_config_path() -> (PathBuf) {
    // let mut is_using_flatpak = false;

    let path_home = home::home_dir().expect("Failed to get home directory");
    let path_config = xdg::BaseDirectories::new().config_home.unwrap_or(path_home.join(".config"));
    
    let mut path_heroic_config = path_config.join("heroic");

    if !path_heroic_config.is_dir() {
        log::info!("Heroic - Attempting to fall back to flatpak");

        // is_using_flatpak = true;
        path_heroic_config = path_home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic");
    }

    // (path_heroic_config, is_using_flatpak)
    path_heroic_config
}