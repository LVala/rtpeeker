use crate::command_line_interface::device_selector::select_device;
use crate::command_line_interface::file_selector::show_pick_file_dialog;
use crate::command_line_interface::start_interface::Action::{AnalyzeFile, CapturePackets};
use std::io::stdin;

pub enum Action {
    CapturePackets(String),
    AnalyzeFile(String),
}

pub fn command_line_interface() -> Action {
    let selected_action;
    loop {
        println!("Welcome to the rtpeeker!");
        println!("Please choose an option:");
        println!("A) Capture packets from a network interface");
        println!("B) Analyze packets from a file");

        let choice = read_line();

        if choice == "A" {
            let device = select_device();
            selected_action = CapturePackets(device);
            break;
        } else if choice == "B" {
            let path = show_pick_file_dialog();
            selected_action = AnalyzeFile(path);
            break;
        } else {
            println!("Invalid choice.\n");
        }
    }
    selected_action
}

fn read_line() -> String {
    let mut choice = String::new();
    stdin().read_line(&mut choice).expect("Failed to read line");
    let choice = choice.trim().to_uppercase();
    choice
}
