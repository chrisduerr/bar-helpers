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
    Command::new("shutdown").args(&["-h", "now"]).spawn().unwrap();
}

// Reboot
fn reboot() {
    Command::new("shutdown").args(&["-r", "now"]).spawn().unwrap();
}

// Check if already running
fn is_running() -> bool {
    let output = Command::new("ps")
                         .args(&["-ax"])
                         .output().unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new("shutdown_menu").unwrap();
    let nbr_running = re.find_iter(&out_str).count();
    (nbr_running >= 2)
}

fn gotta_kill_em_all() {
    Command::new("killall").arg("shutdown_menu").spawn().unwrap();
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
    let wm_name = format!("shutdown_menu-{}", args[1]);

    // Init GTK and Window
    gtk::init().unwrap();
    let window = Window::new(WindowType::Toplevel);
    window.set_title(&wm_name[..]);
    window.set_default_size(125, 125);

    // Shutdown Button
    let font = FontDescription::from_string("Source Sans Pro Bold 12");
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
