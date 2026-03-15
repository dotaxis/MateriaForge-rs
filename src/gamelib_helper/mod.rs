use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, ChildStdout};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[derive(Debug, Clone)]
pub struct Runner {
    pub name: String,
    pub pretty_name: String,
    pub path: PathBuf,
    pub runtime: Option<Runtime>,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub name: String,
    pub pretty_name: String,
    pub path: PathBuf,
}

pub trait Game {
    fn app_id(&self) -> u32;
    fn name(&self) -> &str;
    fn path(&self) -> &Path;
    fn prefix(&self) -> &Path;
    fn runner(&self) -> Option<&Runner>;
}

pub trait PrefixRunner {
    fn run_in_prefix(&self, exe_to_launch: PathBuf, args: Option<Vec<String>>) -> Result<()>;
}

pub trait PrefixedGame: Game + PrefixRunner {}
impl<T: Game + PrefixRunner> PrefixedGame for T {}

pub fn spawn_wine_log_threads(
    stdout: ChildStdout,
    stderr: ChildStderr,
) -> Result<(JoinHandle<()>, JoinHandle<()>)> {
    let wine_log_path = std::env::current_exe()
        .context("Failed to get binary path")?
        .parent()
        .context("Failed to get binary directory")?
        .join("wine.log");
    let wine_log = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&wine_log_path)
            .with_context(|| format!("Failed to create {}", wine_log_path.display()))?,
    ));
    log::info!("Wine logs: {}", wine_log_path.display());

    let log_out = Arc::clone(&wine_log);
    let stdout_handle = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if let Ok(line) = line {
                if let Ok(mut f) = log_out.lock() {
                    let _ = writeln!(f, "{line}");
                }
            }
        }
    });

    let log_err = Arc::clone(&wine_log);
    let stderr_handle = thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            if let Ok(line) = line {
                if let Ok(mut f) = log_err.lock() {
                    let _ = writeln!(f, "{line}");
                }
            }
        }
    });

    Ok((stdout_handle, stderr_handle))
}

pub mod gog_game;
pub mod steam_game;
pub mod steam_lib;
pub mod steam_proton;
