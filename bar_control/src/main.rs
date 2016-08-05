// TODO: Change some stuff from hardcoded to config file
// TODO: If bar ever crashed or makes problems maybe don't just unwrap everything

extern crate time;
extern crate rand;
extern crate i3ipc;
extern crate regex;

use std::thread;
use regex::Regex;
use std::time::Duration;
use i3ipc::I3Connection;
use std::io::prelude::*;
use rand::{thread_rng, Rng};
use std::process::{Command, Stdio};
use std::os::unix::net::UnixStream;
use std::os::unix::io::{FromRawFd, IntoRawFd};


const BG_COL: &'static str = "#121212";
const BG_SEC: &'static str = "#262626";
const FG_COL: &'static str = "#9e9e9e";
const FG_SEC: &'static str = "#616161";
const HL_COL: &'static str = "#702020";
const HL_SEC: &'static str = "#f9a825";
const DISPLAY_COUNT: i32 = 2;
const WORKSPACE_ICONS: [char; 5] = ['', '', '', '', ''];

const VOL_EXEC: &'static str = "$HOME/Scripts/volume_slider";
const POW_EXEC: &'static str = "$HOME/Scripts/shutdown_menu";
const NOT_EXEC: &'static str = "$HOME/Scripts/notification_popup";

struct Screen {
    name: String,
    xres: String,
    xoffset: String
}


fn add_reset(input: &String) -> String {
    format!("{}%{{B-}}%{{F-}}%{{T-}}", input)
}

fn get_ws(screen: &String) -> String {
    let mut conn = I3Connection::connect().unwrap();
    let mut result_str = String::new();
    let workspaces = conn.get_workspaces().unwrap().workspaces;

    for (i, icon) in WORKSPACE_ICONS.iter().enumerate() {
        let mut ws_index: i8 = -1;
        for (x, workspace) in workspaces.iter().enumerate() {
            if &workspace.output == screen {
                let normed_ws_num = (workspace.num - 1) / DISPLAY_COUNT;
                if normed_ws_num == i as i32 {
                    ws_index = x as i8;
                }
            }
        }
        if ws_index == -1 {
            result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ", result_str, BG_COL, BG_SEC, icon);
        }
        else {
            if workspaces[ws_index as usize].visible {
                result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ", result_str, BG_COL, HL_COL, icon);
            }
            else {
                if workspaces[ws_index as usize].urgent {
                    result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ", result_str, BG_COL, HL_SEC, icon);
                }
                else {
                    result_str = format!("{}%{{B{}}}%{{F{}}}  {}  ", result_str, BG_COL, FG_SEC, icon);
                }
            }
        }
    }
    add_reset(&result_str)
}

fn get_date() -> String {
    let curr_time = time::now();
    let curr_time_clock = curr_time.strftime("%H:%M").unwrap();
    add_reset(&format!("%{{B{}}}%{{F{}}}%{{T3}}    {}    ", BG_SEC, FG_COL, curr_time_clock))
}

fn get_not(screen: &String) -> String {
    // Connect to server and check for message
    let mut stream = UnixStream::connect("/tmp/leechnot.sock").unwrap();
    stream.write_all(b"show").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    if response.starts_with("{") {
        let not_script = format!("{} {} &", NOT_EXEC, screen);
        return add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}    %{{A}}", HL_COL, FG_COL, not_script));
    }
    String::new()
}

fn get_vol(screen: &String) -> String {
    let cmd_out = Command::new("amixer").args(&["-D", "pulse", "get", "Master"]).output();
    match cmd_out {
        Ok(out) => {
            let out_str = String::from_utf8_lossy(&out.stdout);
            let vol_end = &out_str[..out_str.find("%").unwrap()];
            let vol = format!("{:>3}", &vol_end[vol_end.rfind("[").unwrap()+1..]);
            let vol_script = format!("{} {} &", VOL_EXEC, screen);
            add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}   {}  %{{A}}", BG_SEC, FG_COL, vol_script, vol))
        },
        Err(_) => String::new(),
    }
}

fn get_pow(screen: &String, icon: &char) -> String {
    let pow_script = format!("{} {} &", POW_EXEC, screen);
    add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}  {}  %{{A}}", BG_SEC, FG_COL, pow_script, icon))
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
    // Stuff I still need to implement
    let barh = 35;
    let font1 = "Source Code Pro Semibold-14";
    let font2 = "FontAwesome-18";
    let pow_icon_choices = ['', '', '', ''];
    let mut rng = thread_rng();
    let pow_icon = rng.choose(&pow_icon_choices).unwrap().clone();

    let mut bar_threads = Vec::new();
    let screens = get_screens();
    for screen in screens.iter() {
        let name = screen.name.clone();
        let xres = screen.xres.clone();
        let xoffset = screen.xoffset.clone();
        bar_threads.push(thread::spawn(move || {
            let rect = format!("{}x{}+{}+0", xres, barh, xoffset);
            let mut lemonbar = Command::new("lemonbar")
                .args(&["-g", &rect[..], "-F", FG_COL, "-B", BG_COL, "-f", font1, "-f", font2])
                .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
            let stdin = lemonbar.stdin.as_mut().unwrap();
            let stdout = lemonbar.stdout.take().unwrap();
            thread::spawn(move || {
                unsafe {
                    let _ = Command::new("sh").stdin(Stdio::from_raw_fd(stdout.into_raw_fd())).spawn();
                }
            });
            loop {
                let date_block = get_date();
                let ws_block = get_ws(&name);
                let not_block = get_not(&name);
                let vol_block = get_vol(&name);
                let pow_block = get_pow(&name, &pow_icon);

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
