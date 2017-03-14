use std::io::prelude::*;
use std::env::home_dir;
use std::fs::File;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub colors: Colors,
    pub exec: Executables,
    pub general: General,
    pub placeholders: Placeholders,
}

#[derive(Deserialize)]
pub struct General {
    pub height: i64,
    pub font: String,
    pub icon_font: String,
    pub power_icon: String,
    pub workspace_icons: String,
}

#[derive(Deserialize)]
pub struct Placeholders {
    pub workspace: String,
    pub general: String,
    pub power: String,
    pub clock: String,
    pub volume: String,
}

#[derive(Deserialize)]
pub struct Executables {
    pub workspace: String,
    pub volume: String,
    pub power: String,
}

#[derive(Deserialize)]
pub struct Colors {
    pub bg_col: String,
    pub bg_sec: String,
    pub fg_col: String,
    pub fg_sec: String,
    pub hl_col: String,
}

pub fn get_config() -> Config {
    let home_path = home_dir().unwrap();
    let home_str = home_path.to_str().unwrap();
    let cfg_path = format!("{}/.config/undeadlemon/config.toml", home_str);

    let mut buf = String::new();
    let mut f = File::open(&cfg_path).unwrap();
    f.read_to_string(&mut buf).unwrap();

    toml::from_str(&buf).unwrap()
}
