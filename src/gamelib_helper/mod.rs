use std::path::{Path, PathBuf};
use anyhow::Result;


#[derive(Debug)]
#[derive(Clone)]
pub struct Runner {
    pub name: String,
    pub pretty_name: String,
    pub path: PathBuf,
    pub runtime: Option<Runtime>,
}

#[derive(Debug)]
#[derive(Clone)]
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

pub trait PrefixLauncher {
    fn run_in_prefix(&self, exe_to_launch: PathBuf, args: Option<Vec<String>>) -> Result<()>;
}

pub mod steam_game;
pub mod gog_game;
pub mod steam_proton;
pub mod steam_lib;