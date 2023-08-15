use crate::command_line_interface::arguments::{
    CommandType, ListInterfaceSubcommand, RtpeekerArgs,
};
use crate::command_line_interface::list_devices::list_devices;
use crate::command_line_interface::start_interface::Action::{AnalyzeFile, CapturePackets};
use clap::Parser;
use std::net::SocketAddr;
use std::process::exit;

pub enum Action {
    CapturePackets(String, SocketAddr),
    AnalyzeFile(String, SocketAddr),
}

pub fn start_command_line_interface() -> Action {
    match RtpeekerArgs::parse().action {
        CommandType::File(args) => {
            if let Ok(socket_addr) = args.socket_to_serve.parse() {
                AnalyzeFile(args.file_path, socket_addr)
            } else {
                parsing_socket_error();
                exit(1);
            }
        }
        CommandType::Interface(args) => {
            if let Ok(socket_addr) = args.socket_to_serve.parse() {
                CapturePackets(args.interface_name, socket_addr)
            } else {
                parsing_socket_error();
                exit(1);
            }
        }
        CommandType::ListInterfaces(sub_command) => {
            let flags = get_flags(sub_command);
            list_devices(flags);
            exit(0);
        }
    }
}

fn get_flags(sub_command: ListInterfaceSubcommand) -> Vec<String> {
    let mut flags = Vec::new();

    if sub_command.filter_running.unwrap_or(false) {
        flags.push("RUNNING".to_string())
    }

    if sub_command.filter_up.unwrap_or(false) {
        flags.push("UP".to_string())
    }

    if sub_command.filter_loopback.unwrap_or(false) {
        flags.push("LOOPBACK".to_string())
    }

    if sub_command.filter_wireless.unwrap_or(false) {
        flags.push("WIRELESS".to_string())
    }
    flags
}

fn parsing_socket_error() {
    println!("Error parsing address! Expected 168.192.1.3:422 or [2001:db8::1]:8080")
}
