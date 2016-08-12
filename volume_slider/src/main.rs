#[macro_use]
extern crate qmlrs;
extern crate regex;

use regex::Regex;
use std::env;
use std::process::Command;


// QTQ Accessible Methods:
struct Volume;
impl Volume {
    fn get_volume(&self) -> i64 {
        let output = Command::new("amixer")
                             .args(&["-D", "pulse", "get", "Master"])
                             .output();
        match output {
            Ok(out) => {
                let stdout_str = String::from_utf8_lossy(&out.stdout);
                let re = Regex::new("\\[([0-9]+)%\\]").unwrap();
                match re.captures(&stdout_str) {
                    Some(caps) => caps.at(1).unwrap().parse().unwrap(),
                    None => 0,
                }
            },
            Err(_) => 0,
        }
    }

    fn set_volume(&self, level: i64) {
        let vol_perc = format!("{}%", level);
        Command::new("amixer")
                .args(&["-q", "-D", "pulse", "set", "Master", &vol_perc[..]])
                .spawn().unwrap();
    }

    fn get_title(&self) -> String {
        let args: Vec<_> = env::args().collect();
        if args.len() <= 1 {
            panic!("Could not find output in command line arguments.");
        }
        format!("volume_slider-{}", args[1])
    }
}

Q_OBJECT! { Volume:
    slot fn get_volume();
    slot fn set_volume(i64);
    slot fn get_title();
}


// Check if Scale already is running
fn is_running() -> bool {
    let output = Command::new("ps")
                         .args(&["-ax"])
                         .output().unwrap();
    let out_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new("[0-9]+:[0-9]+ +[^ ]*volume_slider ").unwrap();
    let nbr_running = re.find_iter(&out_str).count();
    (nbr_running >= 1)
}

fn gotta_kill_em_all() {
    Command::new("killall").arg("volume_slider").spawn().unwrap();
}


// Create a new scale
// If one is already running -> KILL IT
fn main() {
    if is_running() {
        gotta_kill_em_all();
        println!("Already running, closing all instances...");
        return
    }

    // Load QML
    let mut exec_path = env::current_exe().unwrap();
    exec_path.pop();
    let qml_path = format!("{}/volume_slider.qml", exec_path.to_str().unwrap());
    let mut engine = qmlrs::Engine::new();

    engine.set_property("volume", Volume);
    engine.load_local_file(qml_path);

    engine.exec();
}
