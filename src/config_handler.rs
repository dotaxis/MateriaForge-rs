use anyhow::{Context, Result};
use std::{collections::HashMap, env};

static CONFIG_NAME: &str = "MateriaForge.toml";

pub fn write(config: HashMap<&str, String>) -> Result<()> {
    let current_bin = env::current_exe()?;
    let current_dir = current_bin
        .parent()
        .expect("Failed to get current directory");
    let toml_path = current_dir.join(CONFIG_NAME);

    let toml_string = toml::to_string(&config)?;
    std::fs::write(toml_path, toml_string)?;

    Ok(())
}

pub fn read_value(key: &str) -> Result<String> {
    let current_bin = env::current_exe().context("Failed to get binary path")?;
    let current_dir = current_bin
        .parent()
        .context("Failed to get binary directory")?;
    let toml_path = current_dir.join(CONFIG_NAME);

    let toml_string = std::fs::read_to_string(toml_path).context("Couldn't read TOML")?;
    let toml_value: toml::Value =
        toml::from_str(&toml_string).context("Couldn't deserialize TOML")?;

    let value = toml_value
        .get(key)
        .with_context(|| format!("Couldn't find {key} key in {CONFIG_NAME}"))?
        .as_str()
        .with_context(|| format!("{key} value is not a string"))?
        .to_string();

    Ok(value.to_string())
}

pub fn read_value_or_default(key: &str, default: &str) -> String {
    let current_bin = match env::current_exe() {
        Ok(exe) => exe,
        Err(_) => return default.to_string(),
    };
    let current_dir = match current_bin.parent() {
        Some(dir) => dir,
        None => return default.to_string(),
    };
    let toml_path = current_dir.join(CONFIG_NAME);

    let toml_string = match std::fs::read_to_string(&toml_path) {
        Ok(content) => content,
        Err(_) => return default.to_string(),
    };

    let toml_value: toml::Value = match toml::from_str(&toml_string) {
        Ok(value) => value,
        Err(_) => return default.to_string(),
    };

    toml_value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}
