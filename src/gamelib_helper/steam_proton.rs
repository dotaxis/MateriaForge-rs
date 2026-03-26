use anyhow::{bail, Context, Result};
use std::{fs, path::PathBuf};

use crate::gamelib_helper::{Runner, Runtime};

fn get_runtime_appid(manifest_path: PathBuf) -> Result<u32> {
    let manifest_vdf =
        fs::read_to_string(&manifest_path).context("Failed to read manifest file")?;

    keyvalues_parser::parse(&manifest_vdf)
        .context("Failed to parse manifest VDF")?
        .value
        .get_obj()
        .context("No object in VDF")?
        .get("require_tool_appid")
        .context("No require_tool_appid key")?
        .first()
        .context("No require_tool_appid found")?
        .get_str()
        .context("require_tool_appid is not a string")?
        .parse::<u32>()
        .context("Failed to parse require_tool_appid as u32")
}

fn get_runtime(runner: &Runner) -> Result<Runtime> {
    let manifest_path = runner
        .path
        .parent()
        .context("Runner path has no parent")?
        .join("toolmanifest.vdf");
    let runtime_appid = get_runtime_appid(manifest_path)?;
    let steam_dir = steamlocate::SteamDir::locate().context("Failed to locate Steam directory")?;
    let (app, library) = steam_dir
        .find_app(runtime_appid)?
        .with_context(|| format!("Couldn't find runtime app with ID {}", runtime_appid))?;
    let path = library.resolve_app_dir(&app);
    let name = app.name.as_ref().context("No app name?")?.to_string();

    Ok(Runtime {
        name: name.clone(),
        pretty_name: name,
        path,
    })
}

pub fn find_custom_versions(steam_dir: steamlocate::SteamDir) -> Result<Vec<Runner>> {
    let compat_dirs = [
        steam_dir.path().join("compatibilitytools.d"),
        "/usr/share/steam/compatibilitytools.d".into(),
    ];

    let mut custom_runners: Vec<Runner> = Vec::new();
    for path in compat_dirs
        .iter()
        .filter_map(|dir| dir.read_dir().ok())
        .flatten()
        .flatten()
    {
        if path.file_type()?.is_dir() {
            let runner_path = path.path().join("proton");
            if runner_path.is_file() {
                let name = path.file_name().to_string_lossy().to_string();
                let mut runner = Runner {
                    name: name.clone(),
                    pretty_name: name,
                    path: runner_path,
                    runtime: None,
                };
                runner.runtime = get_runtime(&runner).ok();

                custom_runners.push(runner);
            }
        }
    }

    Ok(custom_runners)
}


pub fn find_all_versions(steam_dir: steamlocate::SteamDir) -> Result<Vec<Runner>> {
    let mut proton_versions: Vec<Runner> = Vec::new();
    for library in (steam_dir.libraries()?).flatten() {
        for app in library.apps().flatten() {
            let app_name = app.name.as_ref().context("App name missing.")?;
            if app_name.contains("Proton") {
                let app_path = library.resolve_app_dir(&app).join("proton");
                if app_path.is_file() {
                    let name = app_name
                        .to_lowercase()
                        .split(".")
                        .next()
                        .context("No . found in name")?
                        .replace(" ", "_");

                    let mut runner = Runner {
                        name,
                        pretty_name: app_name.to_string(),
                        path: app_path,
                        runtime: None,
                    };
                    runner.runtime = get_runtime(&runner).ok();

                    proton_versions.push(runner);
                } else {
                    log::info!("Does not contain proton bin: {app_path:?}");
                }
            }
        }
    }

    proton_versions.extend(
        find_custom_versions(steam_dir.clone())
            .context("Failed to find custom Proton versions")?
            .into_iter(),
    );

    if proton_versions.is_empty() {
        bail!("No Proton versions found")
    }

    Ok(proton_versions)
}

pub fn find_highest_version(versions: &[Runner]) -> Option<&Runner> {
    versions.iter().max_by_key(|proton| {
        let pretty_name = &proton.pretty_name;
        let version_parts: Vec<&str> = pretty_name.split_whitespace().collect();
        if version_parts.len() >= 2 && version_parts[0] == "Proton" {
            let version_str = version_parts[1]
                .split('-')
                .next()
                .unwrap_or(version_parts[1]);
            match version_str.parse::<f64>() {
                Ok(n) => ((n * 1000.0) as i64, 0), // Numeric version gets priority
                Err(_) => {
                    // Non-numeric versions like "Experimental" get lower priority
                    if version_str.to_lowercase().contains("experimental") {
                        (0, 2) // Treat "Experimental" as Proton 2.0
                    } else if version_str.to_lowercase().contains("hotfix") {
                        (0, 1) // Treat "Hotfix" as Proton 1.0
                    } else {
                        (0, 0) // Lowest priority for other non-numeric versions
                    }
                }
            }
        } else {
            (0, 0) // Default for unparseable names
        }
    })
}
