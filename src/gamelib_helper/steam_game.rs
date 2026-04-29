use crate::config_handler;
use crate::gamelib_helper::{spawn_wine_log_threads, steam_proton, Game, PrefixRunner, Runner};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SteamGame {
    pub app_id: u32,
    pub name: String,
    pub path: PathBuf,
    pub prefix: PathBuf,
    pub client_path: PathBuf,
    pub runner: Option<Runner>,
}

impl Game for SteamGame {
    fn app_id(&self) -> u32 {
        self.app_id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn path(&self) -> &Path {
        &self.path
    }
    fn prefix(&self) -> &Path {
        &self.prefix
    }
    fn runner(&self) -> Option<&Runner> {
        self.runner.as_ref()
    }
}

impl PrefixRunner for SteamGame {
    fn run_in_prefix(&self, exe_to_launch: PathBuf, args: Option<Vec<String>>) -> Result<()> {
        run_in_prefix(exe_to_launch, self, args)
    }
}

pub fn get_game(app_id: u32, steam_dir: steamlocate::SteamDir) -> Result<SteamGame> {
    let steam_dir_pathbuf = PathBuf::from(steam_dir.path());
    log::info!(
        "Located Steam installation: {}",
        steam_dir_pathbuf.display()
    );
    let (app, library) = steam_dir
        .find_app(app_id)?
        .with_context(|| format!("Couldn't find app with ID {}", app_id))?;
    let path = library.resolve_app_dir(&app);
    let name = app.name.context("No app name?")?.to_string();

    // Search all libraries for compatdata (Steam may place it on a different
    // drive than the game itself, e.g. internal drive vs SD card)
    let prefix = steam_dir
        .libraries()?
        .flatten()
        .map(|lib| lib.path().join(format!("steamapps/compatdata/{app_id}/pfx")))
        .find(|p| p.exists())
        .unwrap_or_else(|| {
            // Fall back to the game's library if compatdata hasn't been created yet
            library
                .path()
                .join(format!("steamapps/compatdata/{app_id}/pfx"))
        });

    let steam_game = SteamGame {
        app_id,
        name,
        path,
        prefix,
        client_path: steam_dir_pathbuf,
        runner: None,
    };

    Ok(steam_game)
}

pub fn select_runner(game: &SteamGame) -> Result<Runner> {
    let steam_dir = steamlocate::SteamDir::from_dir(&game.client_path)?;
    let versions = steam_proton::find_all_versions(steam_dir)?;
    let selected = steam_proton::select_version(&versions)?;
    set_runner(game, &selected.name).ok();
    Ok(selected)
}

pub fn get_runner(game: &SteamGame) -> Result<Runner> {
    let steam_dir = steamlocate::SteamDir::from_dir(&game.client_path)?;

    if let Some(runner_name) = config_handler::read_value("runner").ok() {
        log::info!("Runner specified in config: {runner_name}");
        if let Ok(versions) = steam_proton::find_all_versions(steam_dir.clone()) {
            if let Some(runner) = versions.into_iter().find(|r| r.name == runner_name) {
                return Ok(runner);
            }
        }
    }

    if let Some(tool) = steam_dir
        .compat_tool_mapping()
        .ok()
        .and_then(|m| m.get(&game.app_id).cloned())
    {
        if let Some(tool_name) = tool.name {
            if let Ok(versions) = steam_proton::find_all_versions(steam_dir.clone()) {
                if let Some(runner) = versions.into_iter().find(|r| r.name == tool_name) {
                    return Ok(runner);
                }
            }
        }
    }

    log::info!(
        "No runner configured for app {}, using highest version",
        game.app_id
    );
    let versions = steam_proton::find_all_versions(steam_dir)?;
    steam_proton::find_highest_version(&versions)
        .cloned()
        .context("No Proton versions available")
}

pub fn run_in_prefix(
    exe_to_launch: PathBuf,
    game: &SteamGame,
    args: Option<Vec<String>>,
) -> Result<()> {
    let mut command: Command;

    let proton = game
        .runner
        .clone()
        .with_context(|| format!("Couldn't find proton for game: {game:#?}"))?;
    log::info!("Proton bin: {}", proton.path.display());

    log::info!("Prefix: {}", game.prefix.display());

    // Write steam_appid.txt next to the exe so SteamAPI_Init() can find the
    // app ID even when launched outside a real Steam app session (e.g. non-Steam shortcut).
    if let Some(exe_dir) = exe_to_launch.parent() {
        let appid_file = exe_dir.join("steam_appid.txt");
        if let Err(e) = fs::write(&appid_file, game.app_id.to_string()) {
            log::warn!("Failed to write steam_appid.txt: {e}");
        } else {
            log::info!("Wrote steam_appid.txt: {}", appid_file.display());
        }
    }

    // Build STEAM_COMPAT_MOUNTS
    let ancestor = |path: &Path, levels: usize| -> PathBuf {
        (0..levels)
            .fold(Some(path), |p, _| p.and_then(Path::parent))
            .map(Path::to_path_buf)
            .unwrap_or_default()
    };
    let mut mount_paths: Vec<PathBuf> = vec![
        game.client_path.clone(),
        ancestor(&game.path, 3),
        ancestor(&proton.path, 4),
    ];
    if let Some(runtime) = &proton.runtime {
        mount_paths.push(ancestor(&runtime.path, 4));
    }
    mount_paths.extend(
        std::env::var("STEAM_COMPAT_MOUNTS")
            .unwrap_or_default()
            .split(':')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from),
    );
    mount_paths.dedup();
    let mounts = mount_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(":");

    let install_path = &game
        .path
        .parent()
        .context("Couldn't get parent of game path")?;

    // Trick Proton into making the S: drive contain steamapps
    let library_path = ancestor(&game.path, 3).display().to_string();

    // Allow launching without runtime for runners that don't need one
    command = if let Some(runtime) = &proton.runtime {
        let runtime_path = runtime.path.join("run");
        log::info!("{} path: {runtime_path:?}", runtime.name);
        let mut cmd = Command::new(runtime_path);
        cmd.arg("--").arg(&proton.path);
        cmd
    } else {
        log::info!("No runtime required, launching proton directly");
        Command::new(&proton.path)
    };

    command
        .env("STEAM_COMPAT_MOUNTS", mounts)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &game.client_path)
        .env(
            "STEAM_COMPAT_DATA_PATH",
            game.prefix
                .parent()
                .context("Couldn't get parent of prefix directory")?,
        )
        .env("STEAM_COMPAT_INSTALL_PATH", install_path)
        .env("STEAM_COMPAT_LIBRARY_PATHS", library_path)
        .env("PROTON_SET_GAME_DRIVE", "1")
        .env("WINEDLLOVERRIDES", "dinput=n,b")
        .envs(config_handler::read_env_vars())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("waitforexitandrun")
        .arg(&exe_to_launch);
    let args = args.unwrap_or_default();
    for arg in args {
        log::info!("launch_exe_in_prefix arg: {arg}");
        command.arg(arg);
    }

    let mut child = command.spawn()?;
    log::info!(
        "Launched {}",
        exe_to_launch
            .file_name()
            .context("Couldn't get file_name")?
            .to_string_lossy()
    );

    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

    let (stdout_thread, stderr_thread) = spawn_wine_log_threads(stdout, stderr)?;

    let status = child.wait()?;

    stdout_thread.join().expect("Failed to join stdout thread");
    stderr_thread.join().expect("Failed to join stderr thread");

    if status.success() {
        Ok(log::info!("Process exited successfully"))
    } else {
        bail!("Process exited with an error: {status}");
    }
}

pub fn set_runner(game: &SteamGame, runner: &str) -> Result<()> {
    let re = Regex::new(r#""CompatToolMapping"\s*\{"#)?;
    let replacement = format!(
        r#""CompatToolMapping"
                {{
                    "{}"
                    {{
                        "name"		"{}"
                        "config"		""
                        "priority"		"250"
                    }}
"#,
        &game.app_id, runner
    );

    let remove_re = Regex::new(&format!(
        r#""{}"[^{{]*\{{[^}}]*"name"[^}}]*\}}"#,
        &game.app_id
    ))?;

    let path = &game.client_path.join("config/config.vdf");
    let content = fs::read_to_string(path)?;
    let content = remove_re.replace(&content, "").to_string();
    fs::write(path, re.replace(&content, &replacement).as_bytes())
        .with_context(|| format!("Couldn't write to {path:?}"))?;
    log::info!(
        "Succcessfully set runner for {} to {}",
        &game.app_id,
        runner
    );
    Ok(())
}
