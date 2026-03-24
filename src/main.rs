// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Chase Taylor
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::theme::ColorfulTheme;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use lib_game_detector::{data::SupportedLaunchers, get_detector};
use materia_forge::{
    config_handler,
    gamelib_helper::{self, gog_game, PrefixedGame, DEFAULT_WINEDEBUG},
    logging, resource_handler,
};
use rfd::FileDialog;
use std::{
    collections::HashMap,
    env,
    fmt::Write,
    fs::File,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

const FF7_APPID: u32 = 39140;
const FF7_2026_APPID: u32 = 3837340;
const FF7_GOG_APPID: u32 = 1698970154;

// Check for Steam Deck
static IS_DECK: LazyLock<bool> = LazyLock::new(|| {
    std::fs::read_to_string("/etc/os-release")
        .map(|s| ["SteamOS", "Bazzite"].iter().any(|id| s.contains(id)))
        .unwrap_or(false)
        || env::args().any(|a| a == "-d" || a == "--deck")
});

fn main() {
    if let Err(e) = logging::init("MateriaForge.log") {
        eprintln!("Fatal: {e}");
        std::process::exit(1);
    }

    draw_header();

    if logging::log_and_return(detect_versions()).is_err() {
        std::process::exit(1);
    }
}

fn draw_header() {
    let title = format!("Welcome to MateriaForge {VERSION}");
    let mut description = vec![
        "This script will:",
        "1. Apply patches to FF7's proton prefix to accommodate 7th Heaven",
        "2. Install 7th Heaven to a folder of your choosing",
        "3. Add an app launcher shortcut for 7th Heaven",
        "4. Optionally add a desktop shortcut and Steam shortcut for easy access",
    ];
    let mut footer = [
        "   For support, please open an issue on GitHub,or ask in the #ff7-linux channel of the Tsunamods Discord",
        "",
        "   Use arrow keys and Enter to navigate the prompts.",
    ];

    if *IS_DECK {
        description.append(
            &mut [
                "5. Add a custom controller config for Steam Deck, to allow mouse",
                "   control with trackpad without holding down the STEAM button",
            ]
            .to_vec(),
        );
        footer[2] = "   Use D-Pad and A button to navigate the prompts.";
    }

    // Pad description
    let description: Vec<String> = description
        .iter()
        .map(|line| format!("    {line}    "))
        .collect();

    // Define styles
    let border_style = Style::new().cyan(); // Cyan borders
    let title_style = Style::new().bold().cyan(); // Bold cyan title
    let text_style = Style::new().white(); // White text
    let footer_style = Style::new().dim().white(); // Dim white footer

    // Calculate the maximum line width in the description
    let max_description_width = description.iter().map(|line| line.len()).max().unwrap_or(0);

    // Calculate the banner width based on the longest description line
    let banner_width = max_description_width + 4; // 2 spaces padding + 2 border characters

    // Define border characters
    let top_border = format!("┏{}┓", "━".repeat(banner_width - 2));
    let bottom_border = format!("┗{}┛", "━".repeat(banner_width - 2));
    let middle_border = format!("┣{}┫", "━".repeat(banner_width - 2));
    let border_char = "┃";

    // Print the top border
    println!("{}", border_style.apply_to(top_border));

    // Print the title
    println!(
        "{} {:^max_description_width$} {}",
        border_style.apply_to(border_char),
        title_style.apply_to(title),
        border_style.apply_to(border_char)
    );

    // Print the middle border
    println!("{}", border_style.apply_to(&middle_border));

    // Print the description
    for line in description.iter() {
        println!(
            "{} {:<max_description_width$} {}",
            border_style.apply_to(border_char),
            text_style.apply_to(line),
            border_style.apply_to(border_char)
        );
    }

    // Print the bottom border
    println!("{}", border_style.apply_to(middle_border));

    // Wrap the footer to match the width of the longest description line and print it
    for line in footer.iter() {
        let wrapped_line = textwrap::fill(line, max_description_width);
        let lines: Vec<&str> = wrapped_line.lines().collect();
        for wrapped in &lines {
            println!(
                "{} {:<max_description_width$} {}",
                border_style.apply_to(border_char),
                footer_style.apply_to(wrapped),
                border_style.apply_to(border_char),
            );
        }
        if lines.len() > 1 {
            println!(
                "{} {:<max_description_width$} {}",
                border_style.apply_to(border_char),
                "",
                border_style.apply_to(border_char),
            );
        }
    }

    // Print the bottom border
    println!("{}", border_style.apply_to(bottom_border));
}

fn detect_versions() -> Result<()> {
    let detector = get_detector();
    let ff7_installs = detector.get_all_detected_games();
    let steam_game = ff7_installs.iter().find(|game| {
        game.title.to_lowercase().contains("final fantasy vii")
            && game.source == SupportedLaunchers::Steam
    });
    let heroic_game = ff7_installs.iter().find(|game| {
        game.title.to_lowercase().contains("final fantasy vii")
            && game.source == SupportedLaunchers::HeroicGamesGOG
    });

    let game_version = match (steam_game, heroic_game) {
        (Some(steam), Some(gog)) => {
            let choices = &["Steam", "Heroic Games"];
            let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Multiple versions of FF7 detected. Which one do you want to use?")
                .default(0)
                .items(choices)
                .interact()?;

            match selection {
                0 => steam,
                1 => gog,
                _ => unreachable!(),
            }
        }
        (None, Some(gog)) => {
            log::info!("Heroic Games Launcher install detected!");
            gog
        }
        (Some(steam), None) => {
            log::info!("Steam install detected!");
            steam
        }
        (None, None) => {
            bail!("Couldn't find any supported versions of FF7!");
        }
    };

    run_install(game_version)
}

fn run_install(found_game: &lib_game_detector::data::Game) -> Result<()> {
    let mut config = HashMap::new();
    let game: Box<dyn PrefixedGame>;
    let steam_dir: Option<steamlocate::SteamDir> = gamelib_helper::steam_lib::get_library().ok();

    match found_game.source {
        SupportedLaunchers::Steam => {
            if steam_dir.is_none() {
                bail!("Selected Steam game, but no Steam library could be found?");
            }
            config.insert("type", "steam".to_string());

            let steam_dir = gamelib_helper::steam_lib::get_library()?;
            config.insert("steam_dir", steam_dir.path().display().to_string());

            let (original, remaster) = with_spinner("Finding FF7...", "Done!", || {
                let original =
                    gamelib_helper::steam_game::get_game(FF7_APPID, steam_dir.clone()).ok();
                let remaster =
                    gamelib_helper::steam_game::get_game(FF7_2026_APPID, steam_dir.clone()).ok();
                if original.is_none() && remaster.is_none() {
                    bail!("Couldn't find any supported Steam version of FF7");
                }
                Ok((original, remaster))
            })?;

            game = match (original, remaster) {
                (Some(og), Some(rm)) => {
                    let choices = &[&og.name, &format!("{} (2026)", rm.name)];
                    let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Multiple Steam installations of FF7 were detected. Which one do you want to patch?")
                        .default(0)
                        .items(choices)
                        .interact()
                        .context("Selection failed")?;

                    match selection {
                        0 => Box::new(og),
                        1 => Box::new(rm),
                        _ => unreachable!(),
                    }
                }
                (Some(og), None) => Box::new(og),
                (None, Some(rm)) => Box::new(rm),
                (None, None) => unreachable!(),
            };
        }
        SupportedLaunchers::HeroicGamesGOG => {
            config.insert("type", "gog".to_string());
            game = Box::new(
                gog_game::get_game(FF7_GOG_APPID, &found_game)
                    .context("Failed to get GOG game details")?,
            );
        }
        _ => bail!("Unsupported game selected"),
    }
    config.insert("app_id", game.app_id().to_string());

    let choices = &["Yes", "No"];
    let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to continue installing 7th Heaven?")
        .default(0) // Default to "Yes"
        .items(choices)
        .interact()
        .unwrap();

    if selection == 1 {
        // No
        println!("Understood. Exiting.");
        std::process::exit(0);
    }

    let use_canary = env::args().any(|a| a == "-c" || a == "--canary");
    let update_channel = match use_canary {
        true => "Canary",
        false => "Stable",
    };

    let cache_dir = home::home_dir()
        .context("Couldn't find $HOME?")?
        .join(".cache");

    let exe_path = download_asset("tsunamods-codes/7th-Heaven", cache_dir, use_canary)
        .expect("Failed to download 7th Heaven!");

    if let Some(runner) = &game.runner() {
        log::info!("Runner set for {}: {}", game.name(), runner.pretty_name);
        config.insert("runner", runner.name.clone());
    }

    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("WINEDEBUG", DEFAULT_WINEDEBUG.to_string());

    config_handler::write(config, env_vars).context("Failed to write config")?;

    let install_path = get_install_path()?;
    with_spinner("Installing 7th Heaven...", "Done!", || {
        install_7th(game.as_ref(), exe_path, &install_path, "7thHeaven.log")
    })?;

    with_spinner("Patching installation...", "Done!", || {
        patch_install(&install_path, game.as_ref(), update_channel)
    })?;

    let (steam_shortcut, _) = create_shortcuts(&install_path, steam_dir.clone(), game.app_id())
        .context("Failed to create shortcuts")?;

    add_controller_config(game.as_ref(), &steam_dir, steam_shortcut)
        .context("Failed to set controller config")?;

    println!(
        "{} 7th Heaven successfully installed to '{}'",
        console::style("✔").green(),
        console::style(&install_path.display())
            .bold()
            .underlined()
            .white()
    );

    Ok(())
}

fn download_asset(repo: &str, destination: PathBuf, prerelease: bool) -> Result<PathBuf> {
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

    let exe_asset = assets
        .iter()
        .find(|a| a["name"].as_str().unwrap_or("").ends_with(".exe"))
        .context("No .exe asset found")?;

    let download_url = exe_asset["browser_download_url"]
        .as_str()
        .context("No download URL was passed")?;

    let size = exe_asset["size"].as_u64().unwrap_or(0);

    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", exe_asset["name"]));

    std::fs::create_dir_all(&destination)?;
    let file_name = exe_asset["name"]
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

fn get_install_path() -> Result<PathBuf> {
    let term = console::Term::stdout();
    println!(
        "{} Select a destination for 7th Heaven.",
        console::style("+").yellow()
    );

    loop {
        let install_path = FileDialog::new()
            .set_title("Select Destination")
            .pick_folder();

        if let Some(path) = install_path {
            let path = path.join("7th Heaven");
            let choices = &["Yes", "No"];
            let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Do you want to install 7th Heaven to '{}'?",
                    console::style(path.display()).bold().underlined()
                ))
                .default(0) // Default to "Yes"
                .items(choices)
                .interact()?;

            match confirm {
                0 => {
                    term.clear_last_lines(2)?;
                    std::fs::create_dir_all(&path).with_context(|| {
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

fn install_7th(
    game: &dyn PrefixedGame,
    exe_path: PathBuf,
    install_path: &Path,
    log_file: &str,
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
        format!("/LOG={}", log_file),
    ];

    // let runtime = steam_helper::game::get_game(SLR_APPID, steam_dir)?;

    game.run_in_prefix(exe_path, Some(args))
        .context("Couldn't run 7th Heaven installer")?;

    let current_bin = env::current_exe().context("Failed to get binary path")?;
    let current_dir = current_bin
        .parent()
        .context("Failed to get binary directory")?;
    let toml_path = current_dir.join("MateriaForge.toml");
    std::fs::copy(toml_path, install_path.join("MateriaForge.toml"))
        .context("Failed to copy TOML to install_path")?;

    let launcher_path = if cfg!(debug_assertions) {
        "target/debug/launcher"
    } else {
        "launcher"
    };

    let shortcut_identifier = match game.app_id() {
        FF7_APPID => "(2013)",
        FF7_2026_APPID => "(2026)",
        FF7_GOG_APPID => "(GOG)",
        _ => "(Unknown)",
    };

    std::fs::copy(
        launcher_path,
        install_path.join(format!("Launch 7th Heaven {}", shortcut_identifier)),
    )
    .expect("Failed to copy launcher to install_path");

    Ok(())
}

fn patch_install(install_path: &Path, game: &dyn PrefixedGame, update_channel: &str) -> Result<()> {
    // Send timeout.exe to system32
    let timeout_exe = resource_handler::as_bytes(
        "timeout.exe".to_string(),
        game.prefix().join("drive_c/windows/system32"),
        resource_handler::TIMEOUT_EXE,
    );
    std::fs::write(&timeout_exe.destination, timeout_exe.contents).with_context(|| {
        format!(
            "Couldn't write {} to {:?}",
            timeout_exe.name, timeout_exe.destination
        )
    })?;

    // Patch settings.xml and send to install_path
    let mut settings_xml = resource_handler::as_str(
        "settings.xml".to_string(),
        install_path.join("7thWorkshop"),
        resource_handler::SETTINGS_XML,
    );

    let ff7_version = match game.app_id() {
        FF7_APPID => "Steam",
        FF7_2026_APPID => "SteamReRelease",
        FF7_GOG_APPID => "GOG",
        _ => "Unknown",
    };
    let library_location = &format!(
        "Z:{}",
        install_path
            .join("mods")
            .to_str()
            .unwrap()
            .replace("/", "\\")
    );
    let ff7_exe = match game.app_id() {
        FF7_APPID => "ff7_en.exe",
        FF7_2026_APPID => "FFVII.exe",
        FF7_GOG_APPID => "FFVII.exe",
        _ => "FFVII.exe",
    };
    let ff7_exe_path = &{
        let full = game.path().join(ff7_exe).to_string_lossy().to_string();
        let trimmed = full
            .find("/steamapps/")
            .map_or(full.as_str(), |i| &full[i..]);
        format!("S:{}", trimmed.replace("/", "\\"))
    };

    settings_xml.contents = settings_xml
        .contents
        .replace("LIBRARY_LOCATION", library_location)
        .replace("FF7_EXE", ff7_exe_path)
        .replace("FF7_VERSION", ff7_version)
        .replace("UPDATE_CHANNEL", update_channel);

    std::fs::write(&settings_xml.destination, settings_xml.contents)
        .with_context(|| format!("Couldn't write to {:?}", settings_xml.destination))?;

    // Send dxvk.conf to install_path
    let dxvk_conf = resource_handler::as_str(
        "dxvk.conf".to_string(),
        install_path.to_path_buf(),
        resource_handler::DXVK_CONF,
    );
    std::fs::write(&dxvk_conf.destination, dxvk_conf.contents)
        .with_context(|| format!("Couldn't write to {:?}", dxvk_conf.destination))?;

    Ok(())
}

fn create_shortcuts(
    install_path: &Path,
    steam_dir: Option<steamlocate::SteamDir>,
    app_id: u32,
) -> Result<(bool, ())> {
    // App launcher shortcut
    let applications_dir = xdg::BaseDirectories::new()
        .get_data_home()
        .context("Couldn't get xdg_data_home")?
        .join("applications");

    let shortcut_identifier = match app_id {
        FF7_APPID => "(2013)",
        FF7_2026_APPID => "(2026)",
        FF7_GOG_APPID => "(GOG)",
        _ => "(Unknown)",
    };

    let mut shortcut_file = resource_handler::as_str(
        format!("7th Heaven {}.desktop", shortcut_identifier),
        applications_dir,
        resource_handler::SHORTCUT_FILE,
    );

    shortcut_file.contents = shortcut_file
        .contents
        .replace("INSTALL_PATH", &install_path.to_string_lossy());

    shortcut_file.contents = shortcut_file
        .contents
        .replace("(VER)", &shortcut_identifier);

    std::fs::write(&shortcut_file.destination, &shortcut_file.contents).with_context(|| {
        format!(
            "Couldn't write {} to {:?}",
            shortcut_file.name, shortcut_file.destination
        )
    })?;

    // Icon
    let xdg_cache = xdg::BaseDirectories::new()
        .get_cache_home()
        .context("Couldn't get cache_home")?;
    let logo_png = resource_handler::as_bytes(
        "7th-heaven.png".to_string(),
        xdg_cache,
        resource_handler::LOGO_PNG,
    );
    std::fs::write(&logo_png.destination, &logo_png.contents).with_context(|| {
        format!(
            "Couldn't write {} to {:?}",
            logo_png.name, logo_png.destination
        )
    })?;
    std::process::Command::new("xdg-icon-resource")
        .args([
            "install",
            logo_png.destination.to_str().unwrap(),
            "--size",
            "64",
            "--novendor",
        ])
        .spawn()?
        .wait()?;

    // Desktop shortcut
    let term = console::Term::stdout();
    let choices = &["Yes", "No"];
    let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to add a shortcut to the Desktop?")
        .default(0) // Default to "Yes"
        .items(choices)
        .interact()?;
    match confirm {
        0 => {
            term.clear_last_lines(1)?;
            let desktop_dir = home::home_dir()
                .context("Couldn't get $HOME?")?
                .join("Desktop");
            println!("{} Adding Desktop shortcut.", console::style("!").yellow());
            let desktop_shortcut_path = desktop_dir.join(&shortcut_file.name);
            std::fs::write(&desktop_shortcut_path, shortcut_file.contents).with_context(|| {
                format!(
                    "Couldn't write {} to {:?}",
                    shortcut_file.name, desktop_shortcut_path
                )
            })?;
        }
        _ => {
            term.clear_last_lines(1)?;
        }
    }

    // Non-Steam Game
    let mut steam_shortcut = false;
    if let Some(dir) = steam_dir {
        let choices = &["Yes", "No"];
        let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add a shortcut to Steam?")
            .default(0) // Default to "Yes"
            .items(choices)
            .interact()?;
        match confirm {
            0 => {
                steam_shortcut = true;
                term.clear_last_lines(1)?;
                println!("{} Adding Steam shortcut.", console::style("!").yellow());
                gamelib_helper::steam_lib::add_nonsteam_game(
                    &install_path.join(format!("Launch 7th Heaven {}", shortcut_identifier)),
                    dir,
                )?;
            }
            _ => {
                term.clear_last_lines(1)?;
            }
        }
    }

    Ok((steam_shortcut, ()))
}

fn add_controller_config(
    game: &dyn PrefixedGame,
    steam_dir: &Option<steamlocate::SteamDir>,
    steam_shortcut: bool,
) -> Result<()> {
    if !*IS_DECK {
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
            .default(0) // Default to "Yes"
            .items(choices)
            .interact()?;
        match confirm {
            0 => {
                term.clear_last_lines(1)?;
                println!(
                    "{} Adding controller configuration.",
                    console::style("!").yellow()
                );
                log::info!("Adding controller configuration for Steam Deck.");
            }
            _ => {
                term.clear_last_lines(1)?;
                log::info!("User opted to skip adding controller configuration.");
                return Ok(());
            }
        }

        let controller_vdf = resource_handler::as_str(
            "controller_neptune_gamepad+mouse+click.vdf".to_string(),
            dir.path().join("controller_base/templates/"),
            resource_handler::CONTROLLER_PROFILE,
        );
        std::fs::write(&controller_vdf.destination, controller_vdf.contents).with_context(
            || {
                format!(
                    "Couldn't write {} to {:?}",
                    controller_vdf.name, controller_vdf.destination
                )
            },
        )?;
        gamelib_helper::steam_lib::set_controller_config(dir, game.app_id(), steam_shortcut)?;
    }

    Ok(())
}

fn with_spinner<F, T>(message: &str, success_message: &str, func: F) -> T
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
