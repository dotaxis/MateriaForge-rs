use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const LOGO_PNG: &[u8] = include_bytes!("../resources/logo.png");
pub const TIMEOUT_EXE: &[u8] = include_bytes!("../resources/timeout.exe");

pub const CONTROLLER_PROFILE: &str =
    include_str!("../resources/controller_neptune_gamepad+mouse+click.vdf");
pub const MOD_XML: &str = include_str!("../resources/mod.xml");
pub const SETTINGS_XML: &str = include_str!("../resources/settings.xml");
pub const DXVK_CONF: &str = include_str!("../resources/dxvk.conf");
pub const SHORTCUT_FILE: &str = include_str!("../resources/7th Heaven.desktop");

#[derive(Debug)]
pub struct FileAsStr {
    pub name: String,
    pub destination: PathBuf,
    pub contents: String,
}

#[derive(Debug)]
pub struct FileAsBytes {
    pub name: String,
    pub destination: PathBuf,
    pub contents: Vec<u8>,
}

impl FileAsStr {
    pub fn write(&self) -> Result<()> {
        write_file(&self.name, &self.destination, self.contents.as_bytes())
    }

    pub fn write_to(&self, destination: &Path) -> Result<()> {
        write_file(&self.name, destination, self.contents.as_bytes())
    }
}

impl FileAsBytes {
    pub fn write(&self) -> Result<()> {
        write_file(&self.name, &self.destination, &self.contents)
    }

    pub fn write_if_missing(&self) -> Result<bool> {
        ensure_parent_dir(&self.destination)?;

        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.destination)
        {
            Ok(mut file) => {
                file.write_all(&self.contents).with_context(|| {
                    format!("Couldn't write {} to {:?}", self.name, self.destination)
                })?;
                Ok(true)
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                log::info!(
                    "Skipping {} because it already exists at {:?}",
                    self.name,
                    self.destination
                );
                Ok(false)
            }
            Err(err) => Err(err).with_context(|| {
                format!("Couldn't create {} at {:?}", self.name, self.destination)
            }),
        }
    }
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create destination directory at {:?}", parent))?;
    }

    Ok(())
}

fn write_file(name: &str, destination: &Path, contents: &[u8]) -> Result<()> {
    ensure_parent_dir(destination)?;
    std::fs::write(destination, contents)
        .with_context(|| format!("Couldn't write {} to {:?}", name, destination))
}

pub fn as_bytes(name: String, destination: PathBuf, contents: &[u8]) -> FileAsBytes {
    let full_path = destination.join(&name);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create destination directory");
    }

    FileAsBytes {
        name,
        destination: full_path,
        contents: contents.to_vec(),
    }
}

pub fn as_str(name: String, destination: PathBuf, contents: &str) -> FileAsStr {
    let full_path = destination.join(&name);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create destination directory");
    }

    FileAsStr {
        name,
        destination: full_path,
        contents: contents.to_string(),
    }
}
