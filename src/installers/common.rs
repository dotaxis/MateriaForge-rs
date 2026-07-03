use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rfd::FileDialog;
use std::{
    fmt::Write,
    fs::{self, File},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    gamelib_helper::{self, PrefixedGame},
    resource_handler,
};

use super::{ff7::FF7_GOG_APPID, target::InstallTarget};

pub fn download_asset(
    repo: &str,
    destination: PathBuf,
    prerelease: bool,
    file_pattern: &str,
) -> Result<PathBuf> {
    let client = reqwest::blocking::Client::new();

    let response: serde_json::Value = if prerelease {
        let releases_url = format!("https://api.github.com/repos/{repo}/releases");
        let releases: Vec<serde_json::Value> = client
            .get(&releases_url)
            .header("User-Agent", "rust-client")
            .send()?
            .json()?;
        releases
            .into_iter()
            .next()
            .context("No releases found in GitHub repo")?
    } else {
        let release_url = format!("https://api.github.com/repos/{repo}/releases/latest");
        client
            .get(&release_url)
            .header("User-Agent", "rust-client")
            .send()?
            .json()?
    };

    let assets = response["assets"]
        .as_array()
        .context("No assets found in GitHub link")?;

    let asset = assets
        .iter()
        .find(|a| a["name"].as_str().unwrap_or("").contains(file_pattern))
        .context(format!("No asset found containing '{file_pattern}'"))?;

    let download_url = asset["browser_download_url"]
        .as_str()
        .context("No download URL was passed")?;

    let size = asset["size"].as_u64().unwrap_or(0);

    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", asset["name"]));

    fs::create_dir_all(&destination)?;
    let file_name = asset["name"]
        .as_str()
        .context("Invalid destination file name")?;
    let file_path = destination.join(file_name);

    let mut response = client.get(download_url).send()?;
    let mut file = File::create(&file_path)?;
    let mut writer = pb.wrap_write(&mut file);
    let downloaded = response.copy_to(&mut writer)?;
    pb.set_position(downloaded);
    pb.finish_and_clear();
    pb.println(format!("{} Download complete", console::style("✔").green()));

    Ok(file_path)
}

pub fn get_install_path(target: &dyn InstallTarget) -> Result<PathBuf> {
    let term = console::Term::stdout();
    println!(
        "{} Select a destination for {}.",
        console::style("+").yellow(),
        target.mod_loader_name()
    );

    loop {
        let mut install_path = FileDialog::new()
            .set_title("Select Destination")
            .pick_folder();
        if install_path.is_none() {
            log::warn!("XDG path selection failed, falling back to text input.");
            println!(
                "{} Path selection failed! Please manually enter a destination for {}.",
                console::style("!").yellow(),
                target.mod_loader_name()
            );
            let path: String = Input::new().with_prompt("Path").interact_text()?;
            install_path = Some(PathBuf::from(path));
            term.clear_last_lines(2)?;
        }
        if let Some(path) = install_path {
            let path = path.join(target.install_dir_name());
            let choices = &["Yes", "No"];
            let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Do you want to install {} to '{}'?",
                    target.mod_loader_name(),
                    console::style(path.display()).bold().underlined()
                ))
                .default(0)
                .items(choices)
                .interact()?;

            match confirm {
                0 => {
                    term.clear_last_lines(2)?;
                    fs::create_dir_all(&path).with_context(|| {
                        format!("Couldn't create directory '{}'", path.display())
                    })?;
                    println!(
                        "{} Installing to '{}'",
                        console::style("!").yellow(),
                        console::style(path.display()).bold().underlined().white()
                    );
                    return Ok(path);
                }
                _ => {
                    term.clear_last_lines(1)?;
                    continue;
                }
            }
        }
    }
}

pub fn wine_user_documents_dir(prefix: &Path) -> PathBuf {
    let users_dir = prefix.join("drive_c/users");

    for preferred_user in ["steamuser", "deck", "user"] {
        let candidate = users_dir.join(preferred_user).join("Documents");
        if candidate.is_dir() {
            return candidate;
        }
    }

    if let Ok(entries) = fs::read_dir(&users_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };

            if matches!(name, "Public" | "Default" | "Default User" | "All Users") {
                continue;
            }

            if path.is_dir() {
                return path.join("Documents");
            }
        }
    }

    users_dir.join("steamuser/Documents")
}

pub fn ensure_wine_user_slot(base_dir: &Path, default_user_dir: &str) -> Result<PathBuf> {
    fs::create_dir_all(base_dir).with_context(|| {
        format!(
            "Failed to create game user base directory in Wine prefix: {}",
            base_dir.display()
        )
    })?;

    if let Ok(entries) = fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };

            if path.is_dir() && name.starts_with("user_") {
                return Ok(path);
            }
        }
    }

    let user_dir = base_dir.join(default_user_dir);
    fs::create_dir_all(&user_dir).with_context(|| {
        format!(
            "Failed to create game user directory in Wine prefix: {}",
            user_dir.display()
        )
    })?;

    Ok(user_dir)
}

pub fn install_loader(
    target: &dyn InstallTarget,
    game: &dyn PrefixedGame,
    exe_path: PathBuf,
    install_path: &Path,
) -> Result<()> {
    let exe_path = exe_path
        .canonicalize()
        .with_context(|| format!("Installer not found at {:?}", exe_path))?;

    if let Some(runner) = game.runner() {
        log::info!("Using runner: {} ({})", runner.pretty_name, runner.name);
    } else {
        log::warn!("No runner set on detected game");
    }
    log::info!("Installer path: {}", exe_path.display());
    log::info!("Game prefix: {}", game.prefix().display());

    let args: Vec<String> = vec![
        "/VERYSILENT".to_string(),
        format!(
            "/DIR=Z:{}",
            install_path.to_string_lossy().replace('/', "\\")
        ),
        format!("/LOG={}", target.install_log_name()),
    ];

    game.run_in_prefix(exe_path, Some(args))
        .with_context(|| format!("Couldn't run {} installer", target.mod_loader_name()))?;

    let current_bin = std::env::current_exe().context("Failed to get binary path")?;
    let current_dir = current_bin
        .parent()
        .context("Failed to get binary directory")?;
    let toml_path = current_dir.join("MateriaForge.toml");
    fs::copy(toml_path, install_path.join("MateriaForge.toml"))
        .context("Failed to copy TOML to install_path")?;

    let launcher_path = PathBuf::from(current_dir.join("launcher"));

    let launcher_name = target.launch_binary_with_identifier(game.app_id());

    fs::copy(&launcher_path, install_path.join(launcher_name)).expect(
        format!(
            "Failed to copy launcher from {} to install_path",
            launcher_path.display()
        )
        .as_str(),
    );

    Ok(())
}

pub fn write_common_patch_files(install_path: &Path, game: &dyn PrefixedGame) -> Result<()> {
    let timeout_exe = resource_handler::as_bytes(
        "timeout.exe".to_string(),
        game.prefix().join("drive_c/windows/system32"),
        resource_handler::TIMEOUT_EXE,
    );
    timeout_exe.write_if_missing()?;

    let dxvk_conf = resource_handler::as_str(
        "dxvk.conf".to_string(),
        install_path.to_path_buf(),
        resource_handler::DXVK_CONF,
    );
    dxvk_conf.write()?;

    Ok(())
}

pub fn create_shortcuts(
    target: &dyn InstallTarget,
    install_path: &Path,
    steam_dir: Option<steamlocate::SteamDir>,
    app_id: u32,
) -> Result<bool> {
    let applications_dir = xdg::BaseDirectories::new()
        .get_data_home()
        .context("Couldn't get xdg_data_home")?
        .join("applications");

    let shortcut_identifier = target.shortcut_identifier(app_id);

    let mut shortcut_file = resource_handler::as_str(
        target.desktop_file_name(app_id),
        applications_dir,
        target.desktop_template(),
    );

    shortcut_file.contents = shortcut_file
        .contents
        .replace("INSTALL_PATH", &install_path.to_string_lossy());
    shortcut_file.contents = shortcut_file.contents.replace("(VER)", shortcut_identifier);
    shortcut_file.write()?;

    let xdg_cache = xdg::BaseDirectories::new()
        .get_cache_home()
        .context("Couldn't get cache_home")?;
    let logo_png = resource_handler::as_bytes(
        target.icon_file_name().to_string(),
        xdg_cache,
        target.icon_bytes(),
    );
    logo_png.write()?;

    std::process::Command::new("xdg-icon-resource")
        .args([
            "install",
            logo_png.destination.to_str().unwrap(),
            "--size",
            "64",
            "--novendor",
            "--context",
            "apps",
            "--mode",
            "user",
            target.icon_name(),
        ])
        .spawn()?
        .wait()?;

    let term = console::Term::stdout();
    let choices = &["Yes", "No"];
    let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to add a shortcut to the Desktop?")
        .default(0)
        .items(choices)
        .interact()?;
    if confirm == 0 {
        term.clear_last_lines(1)?;
        let desktop_dir = home::home_dir()
            .context("Couldn't get $HOME?")?
            .join("Desktop");
        println!("{} Adding Desktop shortcut.", console::style("!").yellow());
        let desktop_shortcut_path = desktop_dir.join(&shortcut_file.name);
        shortcut_file.write_to(&desktop_shortcut_path)?;
    } else {
        term.clear_last_lines(1)?;
    }

    let mut steam_shortcut = false;
    if let Some(dir) = steam_dir {
        let choices = &["Yes", "No"];
        let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add a shortcut to Steam?")
            .default(0)
            .items(choices)
            .interact()?;

        if confirm == 0 {
            steam_shortcut = true;
            term.clear_last_lines(1)?;
            println!("{} Adding Steam shortcut.", console::style("!").yellow());
            let launch_binary = target.launch_binary_with_identifier(app_id);
            gamelib_helper::steam_lib::add_nonsteam_game(&install_path.join(launch_binary), dir)?;
        } else {
            term.clear_last_lines(1)?;
        }
    }

    Ok(steam_shortcut)
}

pub fn add_controller_config(
    game: &dyn PrefixedGame,
    steam_dir: &Option<steamlocate::SteamDir>,
    steam_shortcut: bool,
    is_deck: bool,
) -> Result<()> {
    if !is_deck {
        log::info!("Not running on Steam Deck, skipping controller configuration.");
        return Ok(());
    }
    if !steam_shortcut && game.app_id() == FF7_GOG_APPID {
        log::info!("No Steam shortcut added for GOG version, skipping controller configuration.");
        return Ok(());
    }

    let term = console::Term::stdout();
    if let Some(dir) = steam_dir {
        let choices = &["Yes", "No"];
        let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add a controller configuration to Steam?")
            .default(0)
            .items(choices)
            .interact()?;

        if confirm == 0 {
            term.clear_last_lines(1)?;
            println!(
                "{} Adding controller configuration.",
                console::style("!").yellow()
            );
            log::info!("Adding controller configuration for Steam Deck.");
        } else {
            term.clear_last_lines(1)?;
            log::info!("User opted to skip adding controller configuration.");
            return Ok(());
        }

        let controller_vdf = resource_handler::as_str(
            "controller_neptune_gamepad+mouse+click.vdf".to_string(),
            dir.path().join("controller_base/templates/"),
            resource_handler::CONTROLLER_PROFILE,
        );
        controller_vdf.write()?;
        gamelib_helper::steam_lib::set_controller_config(dir, game.app_id(), steam_shortcut)?;
    }

    Ok(())
}

pub fn with_spinner<F, T>(message: &str, success_message: &str, func: F) -> T
where
    F: FnOnce() -> T,
{
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));

    let result = func();
    pb.finish_and_clear();
    println!(
        "{} {} {}",
        console::style("✔").green(),
        message,
        success_message
    );
    result
}
