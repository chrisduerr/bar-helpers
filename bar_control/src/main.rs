extern crate time;
extern crate rand;
extern crate toml;
extern crate i3ipc;
extern crate regex;

mod config;

use std::thread;
use std::io::prelude::*;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::{Command, Stdio};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use regex::Regex;
use i3ipc::I3Connection;
use config::{Config, Executables, Colors};


struct Screen {
    name: String,
    xres: String,
    xoffset: String,
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

        let ws_script = format!("{} {}", exec.ws, i + 1);

        if ws_index.is_none() {
            result_str = format!("{}%{{B{}}}%{{F{}}}%{{A:{}:}}{}{}{}%{{A}}",
                                 result_str,
                                 colors.bg_col,
                                 colors.bg_sec,
                                 ws_script,
                                 config.ws_pad,
                                 icon,
                                 config.ws_pad);
        } else {
            let ws_index = ws_index.unwrap();
            if workspaces[ws_index].visible {
                result_str = format!("{}%{{B{}}}%{{F{}}}%{{A:{}:\
                                      }}{}{}{}%{{A}}",
                                     result_str,
                                     colors.bg_sec,
                                     colors.fg_col,
                                     ws_script,
                                     config.ws_pad,
                                     icon,
                                     config.ws_pad);
            } else if workspaces[ws_index].urgent {
                result_str = format!("{}%{{B{}}}%{{F{}}}%{{A:{}:\
                                      }}{}{}{}%{{A}}",
                                     result_str,
                                     colors.bg_col,
                                     colors.hl_col,
                                     ws_script,
                                     config.ws_pad,
                                     icon,
                                     config.ws_pad);
            } else {
                result_str = format!("{}%{{B{}}}%{{F{}}}%{{A:{}:\
                                      }}{}{}{}%{{A}}",
                                     result_str,
                                     colors.bg_col,
                                     colors.fg_sec,
                                     ws_script,
                                     config.ws_pad,
                                     icon,
                                     config.ws_pad);
            }
        }
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

fn get_vol(screen: &str,
           config: &Config,
           colors: &Colors,
           exec: &Executables)
           -> String {
    let cmd_out = Command::new("amixer")
        .args(&["-D", "pulse", "get", "Master"])
        .output();
    match cmd_out {
        Ok(out) => {
            let out_str = String::from_utf8_lossy(&out.stdout);
            let vol_end = &out_str[..match out_str.find('%') {
                Some(pos) => pos,
                None => return String::new(),
            }];
            let vol = format!("{:>3}",
                              &vol_end[match vol_end.rfind('[') {
                                  Some(pos) => pos,
                                  None => return String::new(),
                              } + 1..]);
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

fn get_pow(screen: &str,
           config: &Config,
           colors: &Colors,
           exec: &Executables)
           -> String {
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
    let screen_re = Regex::new("([a-zA-Z0-9-]*) connected \
                                .*?([0-9]*)x[^+]*\\+([0-9]*)")
        .unwrap();
    for caps in screen_re.captures_iter(&xrandr_str) {
        screens.push(Screen {
            name: caps.at(1).unwrap().to_owned(),
            xres: caps.at(2).unwrap().to_owned(),
            xoffset: caps.at(3).unwrap().to_owned(),
        });
    }
    screens
}

fn i3ipc_get_workspaces(i3con: &mut I3Connection)
                        -> Vec<i3ipc::reply::Workspace> {
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

fn monitor_count_changed(curr_count: &i32) -> bool {
    let output = Command::new("xrandr").output().unwrap();
    let xrandr_str = String::from_utf8_lossy(&output.stdout);
    let monitor_count = xrandr_str.matches(" connected").count() as i32;
    monitor_count != *curr_count
}

fn main() {
    let restart_request = Arc::new(AtomicBool::new(false));
    let screens = get_screens();
    let display_count = screens.len() as i32;

    let mut bar_threads = Vec::new();
    for screen in screens {
        // Load user settings from file
        let mut config = config::get_config();
        let mut colors = config::get_colors();
        let mut exec = config::get_executables();

        // Clone screen props so they're accessible by all threads
        let name = screen.name.clone();
        let xres = screen.xres.clone();
        let xoffset = screen.xoffset.clone();

        // Start i3ipc connection
        let mut i3con = I3Connection::connect().unwrap();

        // Get static pow block
        let mut pow_block = get_pow(&name, &config, &colors, &exec);

        // Start lemonbar
        let rect = format!("{}x{}+{}+0", xres, config.height, xoffset);
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

        // Thread that writes to lemonbar stdin
        let restart_request = restart_request.clone();
        bar_threads.push(thread::spawn(move || {
            let stdin = lemonbar.stdin.as_mut().unwrap();
            let mut update_count = 0;
            loop {
                // Check for config and monitor update every ~5 seconds
                if update_count == 50 {
                    update_count = 0;
                    config = config::get_config();
                    colors = config::get_colors();
                    exec = config::get_executables();
                    pow_block = get_pow(&name, &config, &colors, &exec);

                    if monitor_count_changed(&display_count) {
                        restart_request.store(true, Ordering::SeqCst);
                        break;
                    }
                } else {
                    update_count += 1;
                }

                // Get workspaces from i3ipc, restablish connection if necessary
                let workspaces = i3ipc_get_workspaces(&mut i3con);

                let date_block = get_date(&config, &colors);
                let ws_block = get_ws(&name,
                                      &config,
                                      &colors,
                                      &exec,
                                      &display_count,
                                      &workspaces);
                let vol_block = get_vol(&name, &config, &colors, &exec);

                let bar_string = format!("{}{}{}%{{c}}{}%{{r}}{}{}\n",
                                         pow_block,
                                         config.gen_pad,
                                         ws_block,
                                         date_block,
                                         config.gen_pad,
                                         vol_block);
                let _ = stdin.write((&bar_string[..]).as_bytes());

                thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    for bar_thread in bar_threads {
        let _ = bar_thread.join();
    }

    if restart_request.load(Ordering::SeqCst) {
        main();
    }
}
