use anyhow::{Context, Result};
use dialoguer::theme::ColorfulTheme;
use regex::Regex;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use urlencoding::encode;

pub fn get_library() -> Result<steamlocate::SteamDir> {
    let home_dir = home::home_dir().expect("Couldn't get $HOME?");
    let possible_libraries = vec![
        // Native install directory
        home_dir.join(".steam/root"),
        // Flatpak install directory
        home_dir.join(".var/app/com.valvesoftware.Steam/.steam/root"),
    ];

    let libraries: Vec<PathBuf> = possible_libraries
        .into_iter()
        .filter(|path| path.exists())
        .collect();

    if libraries.len() == 1 {
        let library = steamlocate::SteamDir::from_dir(libraries[0].as_path())
            .context("Couldn't get library")?;
        log::info!("Steam installation located: {}", libraries[0].display());
        return Ok(library);
    }

    log::warn!("Multiple Steam installations detected. Allowing user to select.");
    println!(
        "{} Multiple Steam installations detected.",
        console::style("!").yellow()
    );

    let choices = &[
        format!(
            "Native: {}",
            console::style(libraries[0].display()).bold().underlined()
        ),
        format!(
            "Flatpak: {}",
            console::style(libraries[1].display()).bold().underlined()
        ),
    ];
    let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a Steam installation to continue:")
        .items(choices)
        .default(0)
        .interact()?;

    let library = steamlocate::SteamDir::from_dir(libraries[selection].as_path())
        .context("Failed to get library from dir")?;

    Ok(library)
}

pub fn add_nonsteam_game(file: &Path, steam_dir: steamlocate::SteamDir) -> Result<()> {
    let file_dir = file
        .parent()
        .with_context(|| format!("Couldn't get parent of {file:?}"))?;
    let uid = users::get_current_uid();
    let mut tmp = PathBuf::from("/tmp");
    let mut steam_args: Vec<&str> = vec![];
    let steam_bin = match steam_dir
        .path()
        .to_string_lossy()
        .contains("com.valvesoftware.Steam")
    {
        true => "flatpak",
        _ => "steam",
    };

    // Flatpak Steam
    if steam_bin == "flatpak" {
        steam_args = vec!["run", "com.valvesoftware.Steam"];
        tmp = PathBuf::from(format!(
            "/run/user/{uid}/.flatpak/com.valvesoftware.Steam/tmp"
        ));

        Command::new("flatpak")
            .args([
                "override",
                "--user",
                &format!("--filesystem={}", file_dir.display()),
                "com.valvesoftware.Steam",
            ])
            .status()?;

        Command::new("flatpak")
            .args(["kill", "com.valvesoftware.Steam"])
            .status()?;
    }

    let encoded_url = format!(
        "steam://addnonsteamgame/{}",
        encode(&file.to_string_lossy())
    );
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(tmp.join("addnonsteamgamefile"))?;

    let status = Command::new(steam_bin)
        .args(&steam_args)
        .arg(&encoded_url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        anyhow::bail!("Steam exited with status: {status}");
    }

    log::info!("Added {file:?} to Steam!");
    Ok(())
}

pub fn set_controller_config(
    steam_dir: &steamlocate::SteamDir,
    app_id: u32,
    steam_shortcut: bool,
) -> Result<()> {
    let (shortcut_id, is_gog) = match app_id {
        39140 => ("(2013)", false),
        3837340 => ("(2026)", false),
        1698970154 => ("(GOG)", true),
        _ => ("(Unknown)", false),
    };
    let template = "controller_neptune_gamepad+mouse+click.vdf";
    let config_glob = steam_dir
        .path()
        .join("steamapps/common/Steam Controller Configs/*/config/configset_controller_neptune.vdf")
        .to_string_lossy()
        .to_string();

    let mut entries = Vec::new();
    if !is_gog {
        entries.push(format!(
            r#"	"{app_id}"
	{{
		"template"		"{template}"
	}}"#
        ));
    }
    if steam_shortcut {
        entries.push(format!(
            r#"	"Launch 7th Heaven {shortcut_id}"
	{{
		"template"		"{template}"
	}}"#
        ));
    }
    let new_entries = entries.join("\n");

    // Remove any existing entries for this app ID or the 7th Heaven shortcut
    let remove_app_re = Regex::new(&format!(r#"[ \t]*"{app_id}"[^\{{]*\{{[^\}}]*\}}[ \t]*"#))?;
    let escaped_id = regex::escape(shortcut_id);
    let remove_7h_re = Regex::new(&format!(
        r#"[ \t]*"Launch 7th Heaven {escaped_id}"[^\{{]*\{{[^\}}]*\}}[ \t]*"#
    ))?;
    let blank_lines_re = Regex::new(r"\n([ \t]*\n)+")?;

    let inject_re = Regex::new(r#""controller_config"\s*\{"#)?;
    let replacement = format!("\"controller_config\"\n{{\n{new_entries}");

    for path in glob::glob(&config_glob)
        .context("Invalid glob pattern")?
        .flatten()
    {
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("Couldn't read {:?}", path))?;

        let content = remove_app_re.replace_all(&content, "").to_string();
        let content = remove_7h_re.replace_all(&content, "").to_string();
        let content = blank_lines_re.replace_all(&content, "\n").to_string();
        let content = inject_re.replace(&content, &replacement).to_string();
        std::fs::write(&path, content.as_bytes())
            .with_context(|| format!("Couldn't write to {:?}", path))?;

        log::info!("Patched controller config: {:?}", path);
    }

    Ok(())
}
