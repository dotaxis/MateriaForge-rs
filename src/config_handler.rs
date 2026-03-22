use anyhow::{Context, Result};
use std::{collections::HashMap, env, path::PathBuf};

static CONFIG_NAME: &str = "MateriaForge.toml";

fn config_path() -> Result<PathBuf> {
    let current_bin = env::current_exe().context("Failed to get binary path")?;
    let current_dir = current_bin
        .parent()
        .context("Failed to get binary directory")?;
    Ok(current_dir.join(CONFIG_NAME))
}

pub fn write(config: HashMap<&str, String>, env_vars: HashMap<&str, String>) -> Result<()> {
    let current_bin = env::current_exe()?;
    let current_dir = current_bin
        .parent()
        .expect("Failed to get current directory");
    let toml_path = current_dir.join(CONFIG_NAME);

    let mut table = toml::value::Table::new();
    for (k, v) in &config {
        table.insert(k.to_string(), toml::Value::String(v.clone()));
    }
    if !env_vars.is_empty() {
        let mut env_table = toml::value::Table::new();
        for (k, v) in &env_vars {
            env_table.insert(k.to_string(), toml::Value::String(v.clone()));
        }
        table.insert("env".to_string(), toml::Value::Table(env_table));
    }
    let toml_string = toml::to_string(&table)?;
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

pub fn read_env_vars() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    let Ok(toml_path) = config_path() else {
        return env_vars;
    };
    let Ok(toml_string) = std::fs::read_to_string(toml_path) else {
        return env_vars;
    };
    let Ok(toml_value) = toml_string.parse::<toml::Value>() else {
        return env_vars;
    };
    if let Some(env_table) = toml_value.get("env").and_then(|v| v.as_table()) {
        for (key, value) in env_table {
            if let Some(val_str) = value.as_str() {
                env_vars.insert(key.clone(), val_str.to_string());
            }
        }
    }
    env_vars
}

