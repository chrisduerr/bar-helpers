extern crate gtk;
extern crate gdk;
extern crate toml;
extern crate regex;

use gtk::prelude::*;
use gtk::{StyleContext, CssProvider, Window, WindowType, Box, Scale, Adjustment, Orientation};
use gdk::Screen;
use regex::Regex;
use std::env;
use std::error;
use std::boxed;
use std::fs::File;
use std::io::Read;
use std::process::Command;

fn get_current_volume() -> Result<f64, boxed::Box<error::Error>> {
    let output = Command::new("sh")
        .args(&["-c", "pactl list sinks | grep '^[[:space:]]Volume:' | head -n 1 | tail -n 1 | sed -e 's,.* \\([0-9][0-9]*\\)%.*,\\1,'"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().parse()?)
}

fn set_volume(level: f64) {
    let command = format!("pactl set-sink-volume 0 {}%", level as u8);
    let _ = Command::new("sh").args(&["-c", &command]).spawn();
}

// Check if Scale already is running
fn is_running() -> bool {
    let output = Command::new("ps").args(&["-ax"]).output().unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new("[0-9]+:[0-9]+ [^ ]*volume_slider ").unwrap();
    let nbr_running = re.find_iter(&out_str).count();
    (nbr_running > 1)
}

fn gotta_kill_em_all() {
    let _ = Command::new("pkill").arg("volume_slider").spawn();
}

fn get_position(display: &str) -> (i32, i32) {
    let stdout = Command::new("xrandr").output().unwrap();
    let out = String::from_utf8_lossy(&stdout.stdout);

    let re_string = format!("{}.*? ([0-9]*)x[0-9]*\\+([0-9]*)\\+([0-9]*)", display);
    let re = Regex::new(&re_string[..]).unwrap();
    let caps = re.captures(&out).unwrap();

    let x_width = caps.at(1).unwrap().parse::<i32>().unwrap();
    let x_offset = caps.at(2).unwrap().parse::<i32>().unwrap();
    let y_offset = caps.at(3).unwrap().parse::<i32>().unwrap();

    let x_pos = x_width + x_offset - 350;
    let y_pos = y_offset;

    (x_pos, y_pos)
}

fn get_background_color() -> String {
    let home = env::home_dir().unwrap();
    let home = home.to_str().unwrap();
    let path = format!("{}/.config/undeadlemon/config.toml", &home);

    let mut f = File::open(&path).unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf);

    let tomled: toml::Value = buf.parse().unwrap();
    tomled
        .lookup("colors.bg_col")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

// Create a new scale
// If one is already running -> KILL IT
fn main() {
    while is_running() {
        gotta_kill_em_all();
        return;
    }

    // Check if screen was specified
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        return;
    }

    // Init GTK and Window
    gtk::init().unwrap();
    let window = Window::new(WindowType::Toplevel);
    window.set_title("volume_slider");
    window.set_default_size(250, 30);

    // Create Scale
    let current_vol = get_current_volume().unwrap_or(0.0);
    let adj = Adjustment::new(current_vol, 0.0, 101.0, 1.0, 1.0, 1.0);
    let scale = Scale::new(Orientation::Horizontal, Some(&adj));
    scale.set_draw_value(false);

    // Create Container
    let cont = Box::new(Orientation::Horizontal, 0);
    cont.pack_start(&scale, true, true, 0);

    // Load custom CSS style
    let bg_col = get_background_color();
    let data = format!("scale {{background-color: {};}} scale contents {{padding-left: 5px; \
                        padding-right: 5px;}}",
                       &bg_col);
    let screen = Screen::get_default().unwrap();
    let provider = CssProvider::new();
    let _ = provider.load_from_data(&data);
    StyleContext::add_provider_for_screen(&screen, &provider, 13);

    window.add(&cont);
    window.show_all();

    let win_pos = get_position(&args[1]);
    window.move_(win_pos.0, win_pos.1);

    window.connect_delete_event(|_, _| {
                                    gtk::main_quit();
                                    Inhibit(false)
                                });

    scale.connect_value_changed(move |scale| { set_volume(scale.get_value()); });

    gtk::main();
}
