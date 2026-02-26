use std::{path::{Path, PathBuf}, process::{Command, Stdio}};
use anyhow::{Context, Result};
use serde_json::Value;
use crate::gamelib_helper::{Game, PrefixLauncher, Runner};

#[derive(Debug)]
#[derive(Clone)]
pub struct GogGame {
    pub app_id: u32,
    pub name: String,
    pub path: PathBuf,
    pub prefix: PathBuf,
    pub runner: Option<Runner>,
}

impl Game for GogGame {
    fn app_id(&self) -> u32 { self.app_id }
    fn name(&self) -> &str { &self.name }
    fn path(&self) -> &Path { &self.path }
    fn prefix(&self) -> &Path { &self.prefix }
    fn runner(&self) -> Option<&Runner> { self.runner.as_ref() }
}

impl PrefixLauncher for GogGame {
    fn run_in_prefix(&self, exe_to_launch: PathBuf, args: Option<Vec<String>>) -> Result<()> {
        run_in_prefix(exe_to_launch, self, args)
    }
}

pub fn run_in_prefix(
    exe_to_launch: PathBuf,
    game: &GogGame,
    args: Option<Vec<String>>,
) -> Result<()> {
    let wine = game
        .runner
        .clone()
        .with_context(|| format!("Game has no runner? {game:?}"))?;
    log::info!("Using runner: {}", wine.pretty_name);
    log::info!("Runner bin: {}", wine.path.display());
    log::info!("Wine prefix: {}", game.prefix.display());

    let mut command: Command;
    command = Command::new(wine.path);
    command
        .env("WINEPREFIX", &game.prefix)
        .env("WINEDLLOVERRIDES", "dinput.dll=n,b")
        .stdout(Stdio::null())
        .stderr(Stdio::null()) // TODO: log this properly
        .arg(&exe_to_launch);
    let args = args.unwrap_or_default();
    for arg in args {
        log::info!("run_in_prefix arg: {arg}");
        command.arg(arg);
    }
    command.spawn()?.wait()?;
    log::info!(
        "Launched {}",
        exe_to_launch
            .file_name()
            .context("Couldn't get file_name")?
            .to_string_lossy()
    );

    Ok(())
}

pub fn get_game(app_id: u32, game: &lib_game_detector::data::Game) -> Result<GogGame> {
    let heroic_config_path = get_heroic_config_path();
    let game_json_path = heroic_config_path.join("GamesConfig").join("{}.json}").with_file_name(app_id.to_string() + ".json");
    let game_json_string = std::fs::read_to_string(game_json_path).context("Failed to read GOG game JSON")?;

    let root: Value = serde_json::from_str(&game_json_string).context("Failed to parse GOG game JSON")?;
    let json = root.get(app_id.to_string()).context("GOG game JSON missing app ID key")?;
    let prefix = json.get("winePrefix").and_then(|v| v.as_str()).map(PathBuf::from).context("GOG game JSON missing winePrefix")?;
    let wine_version = json.get("wineVersion").and_then(Value::as_object).context("GOG game JSON missing wineVersion")?;

    // println!("GOG Game JSON: {:#?}", json);

    let mut runner = Runner {
        name: wine_version.get("type").and_then(Value::as_str).map(String::from).context("GOG game JSON missing wineVersion type")?,
        pretty_name: wine_version.get("name").and_then(Value::as_str).map(String::from).context("GOG game JSON missing wineVersion name")?.to_string(),
        path: wine_version.get("bin").and_then(Value::as_str).map(PathBuf::from).context("GOG game JSON missing wineVersion bin")?,
        runtime: None,
    };

    if runner.name == "proton" {
        runner.path = runner.path.parent().unwrap().join("files/bin/wine");
    }

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