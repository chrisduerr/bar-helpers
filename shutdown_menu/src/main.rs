extern crate gtk;
extern crate pango;
extern crate regex;

use gtk::prelude::*;
use gtk::{Window, WindowType, Box, Button, Orientation};
use pango::FontDescription;
use regex::Regex;
use std::env;
use std::process::Command;


// Shutdown
fn shutdown() {
    let _ = Command::new("shutdown").args(&["-h", "now"]).spawn();
}

// Reboot
fn reboot() {
    let _ = Command::new("shutdown").args(&["-r", "now"]).spawn();
}

// Check if already running
fn is_running() -> bool {
    let output = Command::new("ps")
                         .args(&["-ax"])
                         .output().unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new("[0-9]+:[0-9]+ [^ ]*shutdown_menu ").unwrap();
    let nbr_running = re.find_iter(&out_str).count();
    (nbr_running > 1)
}

fn gotta_kill_em_all() {
    let _ = Command::new("killall").arg("shutdown_menu").spawn();
}

fn get_position(display: &String) -> (i32, i32) {
    let stdout = Command::new("xrandr").output().unwrap();
    let out = String::from_utf8_lossy(&stdout.stdout);

    let re_string = format!("{}.*? [0-9]*x[0-9]*\\+([0-9]*)",
                            display);
    let re = Regex::new(&re_string[..]).unwrap();
    let caps = re.captures(&out).unwrap();

    let disp_off = caps.at(1).unwrap().parse::<i32>().unwrap();

    (disp_off, 30)
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
    if args.len() <= 1 {
        return;
    }

    // Init GTK and Window
    gtk::init().unwrap();
    let window = Window::new(WindowType::Toplevel);
    window.set_title("shutdown_menu");
    window.set_default_size(125, 125);

    // Shutdown Button
    let font = FontDescription::from_string("Fira Mono Bold 12");
    let shutdown_btn = Button::new_with_label(" SHUTDOWN ");
    WidgetExt::override_font(&shutdown_btn, Some(&font));
    let shutdown_box = Box::new(Orientation::Horizontal, 0);
    shutdown_box.pack_start(&shutdown_btn, true, true, 10);

    // Restart Button
    let reboot_btn = Button::new_with_label(" REBOOT ");
    WidgetExt::override_font(&reboot_btn, Some(&font));
    let reboot_box = Box::new(Orientation::Horizontal, 0);
    reboot_box.pack_start(&reboot_btn, true, true, 10);

    // Create Container
    let cont = Box::new(Orientation::Vertical, 0);
    cont.pack_start(&shutdown_box, true, true, 10);
    cont.pack_start(&reboot_box, true, true, 10);

    window.add(&cont);
    window.show_all();

    let win_pos = get_position(&args[1]);
    window.move_(win_pos.0, win_pos.1);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    shutdown_btn.connect_clicked(|_| {
        shutdown();
    });
    reboot_btn.connect_clicked(|_| {
        reboot();
    });

    gtk::main();
}
