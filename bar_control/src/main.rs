// TODO: If bar ever crashed or makes problems maybe don't just unwrap everything
// TODO: Use channels and events instead of timed polling
// TODO: Use templates inside config to configure elements

extern crate time;
extern crate rand;
extern crate toml;
extern crate i3ipc;
extern crate regex;

mod config;

use std::thread;
use regex::Regex;
use std::time::Duration;
use i3ipc::I3Connection;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::os::unix::net::UnixStream;
use config::{Config, Executables, Colors};
use std::os::unix::io::{FromRawFd, IntoRawFd};


struct Screen {
    name: String,
    xres: String,
    xoffset: String
}


fn add_reset(input: &String) -> String {
    format!("{}%{{B-}}%{{F-}}%{{T-}}", input)
}

fn get_ws(screen: &String, config: &Config, colors: &Colors, display_count: &i32, i3con: &mut I3Connection) -> String {
    let mut result_str = String::new();
    let workspaces = i3con.get_workspaces().unwrap().workspaces;

    for (i, icon) in config.workspace_icons.chars().enumerate() {
        let mut ws_index: i8 = -1;
        for (x, workspace) in workspaces.iter().enumerate() {
            if &workspace.output == screen {
                let normed_ws_num = (workspace.num - 1) / display_count;
                if normed_ws_num == i as i32 {
                    ws_index = x as i8;
                }
            }
        }
        if ws_index == -1 {
            result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ",
                                 result_str, colors.bg_col, colors.bg_sec, icon);
        }
        else {
            if workspaces[ws_index as usize].visible {
                result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ",
                                     result_str, colors.bg_sec, colors.fg_col, icon);
            }
            else {
                if workspaces[ws_index as usize].urgent {
                    result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ",
                                         result_str, colors.bg_col, colors.hl_col, icon);
                }
                else {
                    result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ",
                                         result_str, colors.bg_col, colors.fg_sec, icon);
                }
            }
        }
    }
    add_reset(&result_str)
}

fn get_date(colors: &Colors) -> String {
    let curr_time = time::now();
    let curr_time_clock = curr_time.strftime("%H:%M").unwrap();
    add_reset(&format!("%{{B{}}}%{{F{}}}%{{T3}}    {}    ",
                       colors.bg_sec, colors.fg_col, curr_time_clock))
}

fn get_not(screen: &String, colors: &Colors, exec: &Executables) -> String {
    // Connect to server and check for message
    let mut stream = UnixStream::connect("/tmp/leechnot.sock").unwrap();
    stream.write_all(b"show").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    if response.starts_with("{") {
        let not_script = format!("{} {} &", exec.not, screen);
        return add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}    %{{A}}",
                                  colors.hl_col, colors.fg_col, not_script));
    }
    String::new()
}

fn get_vol(screen: &String, colors: &Colors, exec: &Executables) -> String {
    let cmd_out = Command::new("amixer").args(&["-D", "pulse", "get", "Master"]).output();
    match cmd_out {
        Ok(out) => {
            let out_str = String::from_utf8_lossy(&out.stdout);
            let vol_end = &out_str[..out_str.find("%").unwrap()];
            let vol = format!("{:>3}", &vol_end[vol_end.rfind("[").unwrap()+1..]);
            let vol_script = format!("{} {} &", exec.vol, screen);
            add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}   {}  %{{A}}",
                               colors.bg_sec, colors.fg_col, vol_script, vol))
        },
        Err(_) => String::new(),
    }
}

fn get_pow(screen: &String, config: &Config, colors: &Colors, exec: &Executables) -> String {
    let pow_script = format!("{} {} &", exec.pow, screen);
    add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}  {}  %{{A}}",
                       colors.bg_sec, colors.fg_col, pow_script, config.power_icon))
}

fn get_screens() -> Vec<Screen> {
    let mut screens = Vec::new();
    let xrandr_out = Command::new("xrandr").output().unwrap();
    let xrandr_str = String::from_utf8_lossy(&xrandr_out.stdout);
    let screen_re = Regex::new("([a-zA-Z0-9-]*) connected ([0-9]*)x[^+]*\\+([0-9]*)").unwrap();
    for caps in screen_re.captures_iter(&xrandr_str) {
        screens.push(Screen {
            name: caps.at(1).unwrap().to_owned(),
            xres: caps.at(2).unwrap().to_owned(),
            xoffset: caps.at(3).unwrap().to_owned()
        });
    }
    screens
}

fn main() {
    let screens = get_screens();
    let display_count = screens.len() as i32;

    let mut bar_threads = Vec::new();
    for screen in screens.iter() {
        let config = config::get_config();
        let colors = config::get_colors();
        let exec = config::get_executables();
        let name = screen.name.clone();
        let xres = screen.xres.clone();
        let xoffset = screen.xoffset.clone();
        let mut i3con = I3Connection::connect().unwrap();
        bar_threads.push(thread::spawn(move || {
            let rect = format!("{}x{}+{}+0", xres, config.height, xoffset);
            let mut lemonbar = Command::new("lemonbar")
                .args(&["-g", &rect[..],
                      "-F", &colors.fg_col[..], "-B", &colors.bg_col[..],
                      "-f", &config.font[..], "-f", &config.icon_font[..]])
                .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
            let stdin = lemonbar.stdin.as_mut().unwrap();
            let stdout = lemonbar.stdout.take().unwrap();

            thread::spawn(move || {
                unsafe {
                    let _ = Command::new("sh").stdin(Stdio::from_raw_fd(stdout.into_raw_fd())).spawn();
                }
            });
            loop {
                let date_block = get_date(&colors);
                let ws_block = get_ws(&name, &config, &colors, &display_count, &mut i3con);
                let not_block = get_not(&name, &colors, &exec);
                let vol_block = get_vol(&name, &colors, &exec);
                let pow_block = get_pow(&name, &config, &colors, &exec);

                let bar_string = format!("{}     {}%{{c}}{}%{{r}}{}     {}\n", pow_block, ws_block, date_block, not_block, vol_block);
                let _ = stdin.write((&bar_string[..]).as_bytes());

                thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    for bar_thread in bar_threads {
        let _ = bar_thread.join();
    }
}
