// TODO: PIPE INTO BASH
// TODO: ADD OFFSET TO ELEMENTS WITHOUT UNDERLINE
// TODO: Fix ugly loop {}
// TODO: Change some stuff from hardcoded to user defined 

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


const BG_COL: &'static str = "#121212";
const BG_SEC: &'static str = "#262626";
const FG_COL: &'static str = "#9e9e9e";
const FG_SEC: &'static str = "#616161";
const HL_COL: &'static str = "#702020";
const DISPLAY_COUNT: i32 = 2;
const WORKSPACE_ICONS: [char; 5] = ['', '', '', '', ''];

const VOL_EXEC: &'static str = "$HOME/Scripts/volume_slider";
const POW_EXEC: &'static str = "$HOME/Scripts/shutdown_menu";
const NOT_EXEC: &'static str = "$HOME/Scripts/notification_popup";

struct Screen {
    name: String,
    xres: String,
    xoffset: String
}


fn add_reset(input: &String) -> String {
    format!("{}%{{B{}}}%{{F{}}}%{{U{}+u}}", input, BG_COL, FG_COL, BG_COL)
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
            result_str = format!("{}%{{B{}}}%{{F{}}}%{{U{}+u}}  {}  ", result_str, BG_COL, FG_SEC, BG_COL, icon);
        }
        else {
            if workspaces[ws_index as usize].visible {
                result_str = format!("{}%{{B{}}}%{{F{}}}%{{U{}+u}}  {}  ", result_str, BG_SEC, FG_COL, HL_COL, icon);
            }
            else {
                if workspaces[ws_index as usize].urgent {
                    result_str = format!("{}%{{B{}}}%{{F{}}}%{{U{}+u}}  {}  ", result_str, BG_COL, HL_COL, BG_SEC, icon);
                }
                else {
                    result_str = format!("{}%{{B{}}}%{{F{}}}%{{U{}+u}}  {}  ", result_str, BG_COL, FG_COL, BG_SEC, icon);
                }
            }
        }
    }
    add_reset(&result_str)
}

fn get_date() -> String {
    let curr_time = time::now();
    let curr_time_fmted = curr_time.strftime("%A, %d. %B %H:%M").unwrap();
    add_reset(&format!("%{{B{}}}%{{U{}+u}}  {}  ", BG_SEC, BG_SEC, curr_time_fmted))
}

fn get_not(screen: &String) -> String {
    // Connect to server and check for message
    let mut stream = UnixStream::connect("/tmp/leechnot.sock").unwrap();
    stream.write_all(b"show").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    if response.starts_with("{") {
        let not_script = format!("{} {} &", NOT_EXEC, screen);
        return add_reset(&format!("%{{B{}}}%{{F{}}}%{{U{}+u}}%{{A:{}:}}    %{{A}}", HL_COL, FG_COL, HL_COL, not_script));
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
            add_reset(&format!("%{{B{}}}%{{F{}}}%{{U{}+u}}%{{A:{}:}}   {}  %{{A}}", BG_SEC, FG_COL, BG_SEC, vol_script, vol))
        },
        Err(_) => String::new(),
    }
}

fn get_pow(screen: &String, icon: &char) -> String {
    let pow_script = format!("{} {} &", POW_EXEC, screen);
    add_reset(&format!("%{{B{}}}%{{F{}}}%{{U{}+u}}%{{A:{}:}}  {}  %{{A}}", BG_SEC, FG_COL, BG_SEC, pow_script, icon))
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
    let ulnh = "3";
    let boff = "-2";
    let font0 = "Source Code Pro Semibold-12";
    let font1 = "FontAwesome-15";
    let pow_icon_choices = ['', '', '', ''];
    let mut rng = thread_rng();
    let pow_icon = rng.choose(&pow_icon_choices).unwrap().clone();

    let screens = get_screens();
    for screen in screens.iter() {
        let name = screen.name.clone();
        let xres = screen.xres.clone();
        let xoffset = screen.xoffset.clone();
        println!("Should start threads here: ");
        thread::spawn(move || {
            println!("Started thread for screen {}", name);
            let rect = format!("{}x{}+{}+0", xres, barh, xoffset);
            let mut lemonbar = Command::new("lemonbar")
                .args(&["-p", "-g", &rect[..], "-o", boff, "-u", ulnh, "-F", FG_COL, "-B", BG_COL, "-U", BG_COL, "-f", font0, "-f", font1])
                .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
            loop {
                let date_block = get_date();
                let ws_block = get_ws(&name);
                let not_block = get_not(&name);
                let vol_block = get_vol(&name);
                let pow_block = get_pow(&name, &pow_icon);

                let bar_string = format!("{}     {}%{{c}}{}%{{r}}{}     {}\n", pow_block, ws_block, date_block, not_block, vol_block);
                lemonbar.stdin.as_mut().unwrap().write((&bar_string[..]).as_bytes()).unwrap();
                lemonbar.stdin.as_mut().unwrap().flush().unwrap();

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    loop {
    }
}
