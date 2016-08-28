extern crate gtk;
extern crate pango;
extern crate regex;
extern crate rustc_serialize;

use gtk::prelude::*;
use gtk::{Window, WindowType, Box, Button, Orientation, Align, Label};
use pango::FontDescription;
use regex::Regex;
use rustc_serialize::json;
use std::env;
use std::process::Command;

static MAX_LINE_LENGTH: usize = 30;


#[derive(RustcDecodable)]
struct Notification {
    body: String,
    app: String,
    summary: String,
}

#[derive(RustcDecodable)]
struct NotificationsList {
    num: u8,
    not: Notification,
}


// Return notifications
fn get_all_nots() -> Vec<NotificationsList> {
    let home = env::home_dir().unwrap();
    let home_str = home.to_str().unwrap();
    let path = format!("{}/Scripts/leechnot.py", home_str);
    println!("{}", path);
    let output = Command::new("python2")
        .args(&[&path[..], "all"])
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);

    // Convert to Decodable str
    let out_str = format!("[{}]", out_str.trim());

    json::decode(&out_str).unwrap()
}

fn delete_not(id: u8) {
    let home_dir = env::home_dir().unwrap();
    let home_dir_str = home_dir.to_str().unwrap();
    let script_dir = format!("{}/Scripts/leechnot.py", home_dir_str);
    let _ = Command::new("python2")
        .args(&[&script_dir[..], "del", &id.to_string()[..]])
        .output()
        .unwrap();
}

fn format_body(mut body: String) -> String {
    if body.len() <= MAX_LINE_LENGTH {
        return body.to_owned();
    }

    let mut body_vec: Vec<String> = Vec::new();
    while body.len() > MAX_LINE_LENGTH {
        if !body.contains(' ') {
            body_vec.push(body[..MAX_LINE_LENGTH].to_owned().clone());
            body = body[MAX_LINE_LENGTH..].to_owned();
        } else {
            let index = body[..MAX_LINE_LENGTH].rfind(' ').unwrap();
            body_vec.push(body[..index].to_owned());
            body = body[index + 1..].to_owned();
        }
    }
    body_vec.push(body);
    body = String::new();

    for part in body_vec {
        body = format!("{}{}\n", body, part);
    }
    (&body[..body.len() - 1]).to_owned()
}

// Check if already running
fn is_running() -> bool {
    let output = Command::new("ps")
        .args(&["-ax"])
        .output()
        .unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new("[0-9]+:[0-9]+ [^ ]*volume_slider ").unwrap();
    let nbr_running = re.find_iter(&out_str).count();
    (nbr_running > 1)
}

fn gotta_kill_em_all() {
    Command::new("killall").arg("notification_popup").spawn().unwrap();
}

fn get_position(display: &str, barh: &str) -> (i32, i32) {
    let stdout = Command::new("xrandr").output().unwrap();
    let out = String::from_utf8_lossy(&stdout.stdout);

    let re_string = format!("{}.*? ([0-9]*)x[0-9]*\\+([0-9]*)\\+([0-9]*)", display);
    let re = Regex::new(&re_string[..]).unwrap();
    let caps = re.captures(&out).unwrap();

    let x_width = caps.at(1).unwrap().parse::<i32>().unwrap();
    let x_offset = caps.at(2).unwrap().parse::<i32>().unwrap();
    let y_offset = caps.at(3).unwrap().parse::<i32>().unwrap();
    let barh = barh.parse::<i32>().unwrap();

    let x_pos = x_offset + x_width - 350;
    let y_pos = y_offset + barh;

    (x_pos, y_pos)
}

fn draw_notifications(cont: &Box, win: &Window) {
    win.resize(350, 1);
    let not_vec = get_all_nots();
    if not_vec.is_empty() {
        std::process::exit(0);
    }

    for not in not_vec {
        // Use App Name as Label
        let _ = not.not.summary;
        let label_str = format!("{}:", not.not.app);
        let label = Label::new(Some(&label_str[..]));
        label.set_margin_top(15);
        label.set_halign(Align::Start);

        let not_but = Button::new_with_label(&format_body(not.not.body.clone())[..]);
        let font = FontDescription::from_string("Fira Mono 12");
        WidgetExt::override_font(&not_but, Some(&font));
        cont.pack_start(&label, true, true, 0);
        cont.pack_start(&not_but, true, true, 0);

        let not_id = not.num;
        let cont_clone = cont.clone();
        let win_clone = win.clone();
        not_but.connect_clicked(move |_| {
            delete_not(not_id);
            for child in cont_clone.get_children() {
                child.destroy();
            }
            draw_notifications(&cont_clone, &win_clone);
        });
    }
    cont.show_all();
}

// Create a new
// If one is already running -> KILL IT
fn main() {
    if is_running() {
        gotta_kill_em_all();
        return;
    }

    // Check if screen was specified
    let args: Vec<_> = env::args().collect();
    if args.len() <= 2 {
        return;
    }

    // Init GTK and Window
    gtk::init().unwrap();
    let window = Window::new(WindowType::Toplevel);
    window.set_title("notification_popup");
    window.set_default_size(350, 0);

    // Create Container
    let cont = Box::new(Orientation::Vertical, 0);
    cont.set_margin_left(10);
    cont.set_margin_right(10);
    cont.set_margin_bottom(15);

    // Create Notifications
    draw_notifications(&cont, &window);

    window.add(&cont);
    window.show_all();

    let win_pos = get_position(&args[1], &args[2]);
    window.move_(win_pos.0, win_pos.1);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
