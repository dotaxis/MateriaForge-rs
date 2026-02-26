use anyhow::{Context, Result};
use console::Style;
use dialoguer::theme::ColorfulTheme;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rfd::FileDialog;
use materia_forge::{
    config_handler, logging, resource_handler,
    steam_helper::{self, game::SteamGame},
    gog_helper,
};
use std::{
    collections::HashMap, env, fmt::Write, fs::File, path::Path, path::PathBuf, time::Duration,
};
use steamlocate::SteamDir;
use lib_game_detector::{data::SupportedLaunchers, get_detector};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const FF7_APPID: u32 = 39140;
const FF7_2026_APPID: u32 = 3837340;
const FF7_GOG_APPID: u32 = 1698970154;

fn main() {
    if let Err(e) = logging::init() {
        eprintln!("Fatal: {e}");
        std::process::exit(1);
    }

    draw_header();

    if logging::log_and_return(detect_versions()).is_err() {
        std::process::exit(1);
    }
}

fn detect_versions() -> Result<()> {
    let detector = get_detector();
    let ff7_installs = detector.get_all_detected_games();
    let steam_game = ff7_installs.iter().find(|game| {
        game.title.to_lowercase().contains("final fantasy vii")
            &&
        game.source == SupportedLaunchers::Steam
    });
    let heroic_game = ff7_installs.iter().find(|game| {
        game.title.to_lowercase().contains("final fantasy vii")
            &&
        game.source == SupportedLaunchers::HeroicGamesGOG
    });

    let _ = match (steam_game, heroic_game) {
        (Some(_), Some(gog)) => {
            let choices = &["Steam", "Heroic Games"];
            let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Multiple versions of FF7 detected. Which one do you want to use?")
                .default(0)
                .items(choices)
                .interact()?;

            match selection {
                0 => seventh_heaven_steam()?,
                1 => seventh_heaven_gog(gog)?,
                _ => unreachable!(),
            }
        }
        (None, Some(gog)) => {
            log::info!("Heroic Games Launcher install detected!");
            seventh_heaven_gog(gog)?
        }
        (Some(_), None) => {
            log::info!("Steam install detected!");
            seventh_heaven_steam()?
        }
        (None, None) => {
            anyhow::bail!("Couldn't find any supported versions of FF7!");
        }
    };
    Ok(())
}

fn draw_header() {
    let title = format!("Welcome to MateriaForge {VERSION}");
    let description = [
        "This script will:",
        "1. Apply patches to FF7's proton prefix to accommodate 7th Heaven",
        "2. Install 7th Heaven to a folder of your choosing",
        "3. Add 7th Heaven to Steam using a custom launcher script",
        "4. Add a custom controller config for Steam Deck, to allow mouse",
        "   control with trackpad without holding down the STEAM button",
    ];
    let footer = "For support, please open an issue on GitHub,or ask in the #ff7-linux channel of the Tsunamods Discord";

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

    // Wrap the footer to match the width of the longest description line
    let wrapped_footer = textwrap::fill(footer, max_description_width); // Wrap the footer text

    // Print the wrapped footer
    for line in wrapped_footer.lines() {
        println!(
            "{} {:<max_description_width$} {}",
            border_style.apply_to(border_char),
            footer_style.apply_to(line),
            border_style.apply_to(border_char)
        );
    }

    // Print the bottom border
    println!("{}", border_style.apply_to(bottom_border));

    let choices = &["Yes", "No"];
    let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to continue?")
        .default(0) // Default to "Yes"
        .items(choices)
        .interact()
        .unwrap();

    if selection == 1 {
        // No
        println!("Understood. Exiting.");
        std::process::exit(0);
    }
}

fn seventh_heaven_steam() -> Result<()> {
    let mut config = HashMap::new();

    let steam_dir = steam_helper::get_library()?;
    config.insert("steam_dir", steam_dir.path().display().to_string());

    let cache_dir = home::home_dir()
        .context("Couldn't find $HOME?")?
        .join(".cache");
    let exe_path = download_asset("tsunamods-codes/7th-Heaven", cache_dir, true)
        .expect("Failed to download 7th Heaven!");

    let game = with_spinner("Finding FF7...", "Done!", || {
        steam_helper::game::get_game(FF7_APPID, steam_dir.clone())
            .or_else(|_| steam_helper::game::get_game(FF7_2026_APPID, steam_dir.clone()))
    })?;

    if let Some(runner) = &game.runner {
        log::info!("Runner set for {}: {}", game.name, runner.pretty_name);
        config.insert("runner", runner.name.clone());
    }

    config_handler::write(config).context("Failed to write config")?;

    // TODO: "Clean install"
    // Wipe common files
    // Verify game files
    // Back up saves
    // Set proton version
    // Wipe prefix
    // Rebuild prefix
    // Restore saves

    let install_path = get_install_path()?;
    with_spinner("Installing 7th Heaven...", "Done!", || {
        install_7th(&game, exe_path, &install_path, "7thHeaven.log")
    })?;

    with_spinner("Patching installation...", "Done!", || {
        patch_install(&install_path, &game)
    })?;

    // TODO: steamOS control scheme + auto-config mod

    create_shortcuts(&install_path, steam_dir).context("Failed to create shortcuts")?;

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

fn seventh_heaven_gog(game: &lib_game_detector::data::Game) -> Result<()> {
    let gog_game = gog_helper::get_game(FF7_GOG_APPID, game).context("Failed to get GOG game details")?;
    println!("GOG Game details: {:#?}", gog_game);
    println!("Runner details: {:#?}", gog_game.runner);
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
                    std::fs::create_dir_all(&path)
                        .with_context(|| format!("Couldn't create directory '{}'", path.display()))?;
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
    game: &SteamGame,
    exe_path: PathBuf,
    install_path: &Path,
    log_file: &str,
) -> Result<()> {
    let exe_path = exe_path
        .canonicalize()
        .with_context(|| format!("Installer not found at {:?}", exe_path))?;

    if let Some(runner) = &game.runner {
        log::info!("Using runner: {} ({})", runner.pretty_name, runner.name);
    } else {
        log::warn!("No runner set on detected game");
    }
    log::info!("Installer path: {}", exe_path.display());
    log::info!("Game prefix: {}", game.prefix.display());

    let args: Vec<String> = vec![
        "/VERYSILENT".to_string(),
        format!(
            "/DIR=Z:{}",
            install_path.to_string_lossy().replace('/', "\\")
        ),
        format!("/LOG={}", log_file),
    ];

    // let runtime = steam_helper::game::get_game(SLR_APPID, steam_dir)?;

    steam_helper::game::launch_exe_in_prefix(exe_path, game, Some(args))
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
    std::fs::copy(launcher_path, install_path.join("Launch 7th Heaven"))
        .expect("Failed to copy launcher to install_path");

    Ok(())
}

fn patch_install(install_path: &Path, game: &SteamGame) -> Result<()> {
    // Send timeout.exe to system32
    let timeout_exe = resource_handler::as_bytes(
        "timeout.exe".to_string(),
        game.prefix.join("drive_c/windows/system32"),
        resource_handler::TIMEOUT_EXE,
    );
    std::fs::write(&timeout_exe.destination, timeout_exe.contents).with_context(|| {
        format!(
            "Couldn't write {} to {:?}",
            timeout_exe.name, timeout_exe.destination
        )
    })?;

    // Send Default.xml to install path
    let default_xml = resource_handler::as_str(
        "Default.xml".to_string(),
        install_path.join("7thWorkshop/profiles"),
        resource_handler::DEFAULT_XML,
    );
    std::fs::write(&default_xml.destination, default_xml.contents).with_context(|| {
        format!(
            "Couldn't write {} to {:?}",
            default_xml.name, default_xml.destination
        )
    })?;

    // Patch settings.xml and send to install_path
    let mut settings_xml = resource_handler::as_str(
        "settings.xml".to_string(),
        install_path.join("7thWorkshop"),
        resource_handler::SETTINGS_XML,
    );

    let library_location = &format!(
        "Z:{}",
        install_path
            .join("mods")
            .to_str()
            .unwrap()
            .replace("/", "\\")
    );
    let ff7_exe = match game.app_id  {
        FF7_APPID => "ff7_en.exe",
        FF7_2026_APPID => "FFVII.exe",
        _ => "ff7_en.exe"
    };
    let ff7_exe_path = &format!(
        "Z:{}",
        game.path
            .join(ff7_exe)
            .to_string_lossy()
            .replace("/", "\\")
    );
    let update_channel = match game.app_id  {
        FF7_APPID => "Stable",
        FF7_2026_APPID => "Canary",
        _ => "Stable"
    };


    settings_xml.contents = settings_xml
        .contents
        .replace("LIBRARY_LOCATION", library_location)
        .replace("FF7_EXE", ff7_exe_path)
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

fn create_shortcuts(install_path: &Path, steam_dir: SteamDir) -> Result<()> {
    // App launcher shortcut
    let applications_dir = xdg::BaseDirectories::new().get_data_home().context("Couldn't get xdg_data_home")?.join("applications");
    let mut shortcut_file = resource_handler::as_str(
        "7th Heaven.desktop".to_string(),
        applications_dir,
        resource_handler::SHORTCUT_FILE,
    );

    shortcut_file.contents = shortcut_file
        .contents
        .replace("INSTALL_PATH", &install_path.to_string_lossy());

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
    let choices = &["Yes", "No"];
    let confirm = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to add a shortcut to Steam?")
        .default(0) // Default to "Yes"
        .items(choices)
        .interact()?;
    match confirm {
        0 => {
            term.clear_last_lines(1)?;
            println!("{} Adding Steam shortcut.", console::style("!").yellow());
            steam_helper::add_nonsteam_game(&install_path.join("Launch 7th Heaven"), steam_dir)?;
        }
        _ => {
            term.clear_last_lines(1)?;
        }
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
