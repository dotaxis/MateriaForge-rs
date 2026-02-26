use std::path::PathBuf;
use anyhow::{Context, Result};
use serde_json::Value;
use lib_game_detector::data::Game;

#[derive(Debug)]
pub struct GogGame {
    pub app_id: u32,
    pub name: String,
    pub path: PathBuf,
    pub prefix: PathBuf,
    pub runner: Option<Runner>,
}

#[derive(Debug)]
pub struct Runner {
    pub name: String,
    pub runner_type: String,
    pub path: PathBuf,
}

pub fn get_game(app_id: u32, game: &Game) -> Result<GogGame> {
    let heroic_config_path = get_heroic_config_path();
    let game_json_path = heroic_config_path.join("GamesConfig").join("{}.json}").with_file_name(app_id.to_string() + ".json");
    let game_json_string = std::fs::read_to_string(game_json_path).context("Failed to read GOG game JSON")?;

    let root: Value = serde_json::from_str(&game_json_string).context("Failed to parse GOG game JSON")?;
    let json = root.get(app_id.to_string()).context("GOG game JSON missing app ID key")?;
    let prefix = json.get("winePrefix").and_then(|v| v.as_str()).map(PathBuf::from).context("GOG game JSON missing winePrefix")?;
    let wine_version = json.get("wineVersion").and_then(Value::as_object).context("GOG game JSON missing wineVersion")?;

    // println!("GOG Game JSON: {:#?}", json);

    let runner = Runner {
        name: wine_version.get("name").and_then(Value::as_str).map(String::from).context("GOG game JSON missing wineVersion name")?.to_string(),
        runner_type: wine_version.get("type").and_then(Value::as_str).map(String::from).context("GOG game JSON missing wineVersion type")?,
        path: wine_version.get("bin").and_then(Value::as_str).map(PathBuf::from).context("GOG game JSON missing wineVersion bin")?,
    };

    Ok(GogGame {
        app_id,
        name: game.title.clone(),
        path: game.path_game_dir.clone().ok_or_else(|| anyhow::anyhow!("Game is missing path_game_dir in detector result"))?,
        prefix: prefix,
        runner: Some(runner),
    })
}

/// Get path to the Heroic Games Launcher config dir, falling back to the flatpak version if necessary
fn get_heroic_config_path() -> PathBuf {
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