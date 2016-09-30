extern crate time;
extern crate rand;
extern crate toml;
extern crate i3ipc;
extern crate regex;
extern crate libudev;

mod config;

use std::thread;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use regex::Regex;
use i3ipc::I3Connection;
use config::{Config, Executables, Colors};
use libudev::{Context, Monitor};
use time::Duration;


struct Screen {
    name: String,
    xres: String,
    xoffset: String,
}

struct Lemonbar {
    bar: std::process::Child,
    screen: Screen,
    pow_block: String,
}


fn add_reset(input: &str) -> String {
    format!("{}%{{B-}}%{{F-}}%{{T-}}", input)
}

fn get_ws(screen: &str,
          config: &Config,
          colors: &Colors,
          exec: &Executables,
          display_count: &i32,
          workspaces: &[i3ipc::reply::Workspace])
          -> String {
    let mut result_str = String::new();

    for (i, icon) in config.workspace_icons.chars().enumerate() {
        let mut ws_index = None;
        for (x, workspace) in workspaces.iter().enumerate() {
            if &workspace.output == screen {
                let normed_ws_num = (workspace.num - 1) / display_count;
                if normed_ws_num == i as i32 {
                    ws_index = Some(x);
                }
            }
        }

        let (col_prim, col_sec) = match ws_index {
            None => (&colors.bg_col, &colors.bg_sec),
            Some(i) => {
                if workspaces[i].visible {
                    (&colors.bg_sec, &colors.fg_col)
                } else if workspaces[i].urgent {
                    (&colors.bg_col, &colors.hl_col)
                } else {
                    (&colors.bg_col, &colors.fg_sec)
                }
            }
        };

        let ws_script = format!("{} {}", exec.ws, i + 1);
        result_str = format!("{}%{{B{}}}%{{F{}}}%{{A:{}:}}{}{}{}%{{A}}",
                             result_str,
                             col_prim,
                             col_sec,
                             ws_script,
                             config.ws_pad,
                             icon,
                             config.ws_pad);
    }

    add_reset(&result_str)
}

fn get_date(config: &Config, colors: &Colors) -> String {
    let curr_time = time::now();
    let curr_time_clock = match curr_time.strftime("%H:%M") {
        Ok(fmt) => fmt,
        Err(_) => return String::new(),
    };

    add_reset(&format!("%{{B{}}}%{{F{}}}{}{}{}",
                       colors.bg_sec,
                       colors.fg_col,
                       config.dat_pad,
                       curr_time_clock,
                       config.dat_pad))
}

fn get_vol(screen: &str, config: &Config, colors: &Colors, exec: &Executables) -> String {
    let cmd_out = Command::new("amixer")
        .args(&["-D", "pulse", "get", "Master"])
        .output();

    match cmd_out {
        Ok(out) => {
            let out_str = String::from_utf8_lossy(&out.stdout);
            let vol_reg = Regex::new(".*\\[([0-9]*)%\\]").unwrap();
            let vol = match vol_reg.captures(&out_str) {
                Some(caps) => {
                    match caps.at(1) {
                        Some(vol) => format!("{:>3}", vol),
                        None => String::new(),
                    }
                }
                None => String::new(),
            };

            let vol_script = format!("{} {} &", exec.vol, screen);

            add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}{} {}{}%{{A}}",
                               colors.bg_sec,
                               colors.fg_col,
                               vol_script,
                               config.vol_pad,
                               vol,
                               config.vol_pad))
        }
        Err(_) => String::new(),
    }
}

fn get_pow(screen: &str, config: &Config, colors: &Colors, exec: &Executables) -> String {
    let pow_script = format!("{} {} &", exec.pow, screen);

    add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}{}{}{}%{{A}}",
                       colors.bg_sec,
                       colors.fg_col,
                       pow_script,
                       config.pow_pad,
                       config.power_icon,
                       config.pow_pad))
}

fn get_screens() -> Vec<Screen> {
    let mut screens = Vec::new();

    let xrandr_out = match Command::new("xrandr").output() {
        Ok(out) => out,
        Err(_) => return Vec::new(),
    };

    let xrandr_str = String::from_utf8_lossy(&xrandr_out.stdout);
    let screen_re = Regex::new("([a-zA-Z0-9-]*) connected .*?([0-9]*)x[^+]*\\+([0-9]*)").unwrap();

    for caps in screen_re.captures_iter(&xrandr_str) {
        screens.push(Screen {
            name: caps.at(1).unwrap().to_owned(),
            xres: caps.at(2).unwrap().to_owned(),
            xoffset: caps.at(3).unwrap().to_owned(),
        });
    }

    screens
}

fn i3ipc_get_workspaces(i3con: &mut I3Connection) -> Vec<i3ipc::reply::Workspace> {
    match i3con.get_workspaces() {
        Ok(gw) => gw.workspaces,
        Err(_) => {
            *i3con = match I3Connection::connect() {
                Ok(i3c) => i3c,
                Err(_) => return Vec::new(),
            };
            match i3con.get_workspaces() {
                Ok(gw) => gw.workspaces,
                Err(_) => Vec::new(),
            }
        }
    }
}

fn main() {
    loop {
        let screens = get_screens();
        let display_count = screens.len() as i32;

        let mut config = config::get_config();
        let mut colors = config::get_colors();
        let mut exec = config::get_executables();

        let mut lemonbars = Vec::new();

        let mut i3con = I3Connection::connect().unwrap();

        for screen in screens {
            // Start lemonbar
            let rect = format!("{}x{}+{}+0", screen.xres, config.height, screen.xoffset);
            let mut lemonbar = Command::new("lemonbar")
                .args(&["-g",
                        &rect[..],
                        "-F",
                        &colors.fg_col[..],
                        "-B",
                        &colors.bg_col[..],
                        "-f",
                        &config.font[..],
                        "-f",
                        &config.icon_font[..]])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            // Thread that controls executing lemonbar stdout
            let stdout = lemonbar.stdout.take().unwrap();
            thread::spawn(move || {
                unsafe {
                    let _ = Command::new("sh")
                        .stdin(Stdio::from_raw_fd(stdout.into_raw_fd()))
                        .spawn();
                }
            });

            // Collect all lemonbars in one vector for future processing
            let pow = get_pow(&screen.name, &config, &colors, &exec);
            let lemonstruct = Lemonbar {
                bar: lemonbar,
                screen: screen,
                pow_block: pow,
            };
            lemonbars.push(lemonstruct);
        }

        // Setup listener for monitor count change
        let context = Context::new().unwrap();
        let mut monitor = Monitor::new(&context).unwrap();
        monitor.match_subsystem("drm").unwrap();
        let mut socket = monitor.listen().unwrap();

        let mut curr_time = time::now();
        loop {
            let elapsed = time::now() - curr_time;
            if elapsed >= Duration::seconds(3) {

                curr_time = time::now();
                config = config::get_config();
                colors = config::get_colors();
                exec = config::get_executables();

                for lemonbar in &mut lemonbars {
                    lemonbar.pow_block = get_pow(&lemonbar.screen.name, &config, &colors, &exec);
                }

                // Kill all bars and restart on monitor change
                if socket.receive_event().is_some() {
                    for lemonbar in &mut lemonbars {
                        let _ = lemonbar.bar.kill();
                    }
                    break;
                }
            }

            let workspaces = i3ipc_get_workspaces(&mut i3con);
            let date_block = get_date(&config, &colors);

            for lemonbar in &mut lemonbars {
                let stdin = lemonbar.bar.stdin.as_mut().unwrap();

                let ws_block = get_ws(&lemonbar.screen.name,
                                      &config,
                                      &colors,
                                      &exec,
                                      &display_count,
                                      &workspaces);
                let vol_block = get_vol(&lemonbar.screen.name, &config, &colors, &exec);

                let bar_string = format!("{}{}{}%{{c}}{}%{{r}}{}{}\n",
                                         lemonbar.pow_block,
                                         config.gen_pad,
                                         ws_block,
                                         date_block,
                                         config.gen_pad,
                                         vol_block);

                let _ = stdin.write((&bar_string[..]).as_bytes());
            }
            thread::sleep(Duration::milliseconds(100).to_std().unwrap());
        }
    }
}
