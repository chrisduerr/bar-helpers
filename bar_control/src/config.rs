use toml;
use std::fs::File;
use std::io::prelude::*;
use rand::{thread_rng, Rng};


pub struct Config {
    pub height: i64,
    pub power_icon: char,
    pub font: String,
    pub icon_font: String,
    pub workspace_icons: String
}

pub struct Executables {
    pub pow: String,
    pub vol: String,
    pub not: String
}

pub struct Colors {
    pub bg_col: String,
    pub bg_sec: String,
    pub fg_col: String,
    pub fg_sec: String,
    pub hl_col: String
}


pub fn get_value(toml: &toml::Value, value: &str) -> toml::Value {
    toml.lookup(value).unwrap().clone()
}

pub fn get_executables() -> Executables {
    let mut f = File::open("config.toml").unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf);

    let config: toml::Value = buf.parse().unwrap();
    Executables {
        pow: get_value(&config, "exec.power").as_str().unwrap().to_owned(),
        vol: get_value(&config, "exec.volume").as_str().unwrap().to_owned(),
        not: get_value(&config, "exec.notifications").as_str().unwrap().to_owned(),
    }
}

pub fn get_colors() -> Colors {
    let mut f = File::open("config.toml").unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf);

    let config: toml::Value = buf.parse().unwrap();
    Colors {
        bg_col: get_value(&config, "colors.background_color").as_str().unwrap().to_owned(),
        bg_sec: get_value(&config, "colors.background_secondary").as_str().unwrap().to_owned(),
        fg_col: get_value(&config, "colors.foreground_color").as_str().unwrap().to_owned(),
        fg_sec: get_value(&config, "colors.foreground_secondary").as_str().unwrap().to_owned(),
        hl_col: get_value(&config, "colors.highlight_color").as_str().unwrap().to_owned()
    }
}

pub fn get_config() -> Config {
    let mut f = File::open("config.toml").unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf);

    let config: toml::Value = buf.parse().unwrap();

    // Pick one random pow icon
    let mut rng = thread_rng();
    let pow_icon_choices: Vec<char> = get_value(&config, "general.power_icons").as_str().unwrap().chars().collect();
    let pow_icon = rng.choose(&pow_icon_choices).unwrap();

    Config {
        height: get_value(&config, "general.height").as_integer().unwrap(),
        power_icon: pow_icon.clone(),
        font: get_value(&config, "general.font").as_str().unwrap().to_owned(),
        icon_font: get_value(&config, "general.icon_font").as_str().unwrap().to_owned(),
        workspace_icons: get_value(&config, "general.workspace_icons").as_str().unwrap().to_owned()
    }
}
