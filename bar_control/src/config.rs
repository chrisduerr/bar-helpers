use toml;
use std::env::home_dir;
use std::fs::File;
use std::io::prelude::*;


pub struct Config {
    pub height: i64,
    pub font: String,
    pub ws_pad: String,
    pub gen_pad: String,
    pub pow_pad: String,
    pub dat_pad: String,
    pub vol_pad: String,
    pub icon_font: String,
    pub power_icon: String,
    pub workspace_icons: String,
}

pub struct Executables {
    pub ws: String,
    pub pow: String,
    pub vol: String,
}

pub struct Colors {
    pub bg_col: String,
    pub bg_sec: String,
    pub fg_col: String,
    pub fg_sec: String,
    pub hl_col: String,
}


pub fn get_value_str(toml: &toml::Value, value: &str) -> String {
    let tml_val = toml.lookup(value).unwrap();
    tml_val.as_str().unwrap().to_string()
}

pub fn get_value_int(toml: &toml::Value, value: &str) -> i64 {
    let tml_val = toml.lookup(value).unwrap().clone();
    tml_val.as_integer().unwrap()
}

pub fn get_config_toml() -> toml::Value {
    let home_path = home_dir().unwrap();
    let home_str = home_path.to_str().unwrap();
    let cfg_path = format!("{}/.config/undeadlemon/config.toml", home_str);

    let mut buf = String::new();
    let mut f = File::open(&cfg_path).unwrap();
    f.read_to_string(&mut buf).unwrap();

    buf.parse().unwrap()
}

pub fn get_executables() -> Executables {
    let config: toml::Value = get_config_toml();
    Executables {
        pow: get_value_str(&config, "exec.power"),
        vol: get_value_str(&config, "exec.volume"),
        ws: get_value_str(&config, "exec.switch_focused_workspace"),
    }
}

pub fn get_colors() -> Colors {
    let config: toml::Value = get_config_toml();
    Colors {
        hl_col: get_value_str(&config, "colors.highlight_color"),
        bg_col: get_value_str(&config, "colors.background_color"),
        fg_col: get_value_str(&config, "colors.foreground_color"),
        bg_sec: get_value_str(&config, "colors.background_secondary"),
        fg_sec: get_value_str(&config, "colors.foreground_secondary"),
    }
}

pub fn get_config() -> Config {
    let config: toml::Value = get_config_toml();
    Config {
        font: get_value_str(&config, "general.font"),
        height: get_value_int(&config, "general.height"),
        pow_pad: get_value_str(&config, "placeholders.power"),
        dat_pad: get_value_str(&config, "placeholders.clock"),
        vol_pad: get_value_str(&config, "placeholders.volume"),
        gen_pad: get_value_str(&config, "placeholders.general"),
        ws_pad: get_value_str(&config, "placeholders.workspace"),
        icon_font: get_value_str(&config, "general.icon_font"),
        power_icon: get_value_str(&config, "general.power_icon"),
        workspace_icons: get_value_str(&config, "general.workspace_icons"),
    }
}
