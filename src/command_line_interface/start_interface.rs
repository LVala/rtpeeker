use crate::command_line_interface::device_selector::list_devices;
use crate::command_line_interface::start_interface::Action::{AnalyzeFile, CapturePackets};
use clap::{App, Arg, ArgMatches, SubCommand};
use std::net::SocketAddr;
use std::process::exit;

pub enum Action {
    CapturePackets(String, SocketAddr),
    AnalyzeFile(String, SocketAddr),
}

pub fn command_line_interface() -> Action {
    let args = get_args();

    if let Some(sub_args) = args.subcommand_matches("list") {
        let mut flags = Vec::new();

        if sub_args.is_present("up") {
            flags.push("UP".to_string());
        }

        if sub_args.is_present("running") {
            flags.push("RUNNING".to_string());
        }

        if sub_args.is_present("loopback") {
            flags.push("LOOPBACK".to_string());
        }

        if sub_args.is_present("wireless") {
            flags.push("WIRELESS".to_string());
        }
        list_devices(flags);
        exit(0);
    }

    if let Some(interface) = args.value_of("interface") {
        let socket = args.value_of("socket").unwrap_or("");

        if let Ok(socket_addr) = socket.parse() {
            return CapturePackets(interface.to_string(), socket_addr);
        } else {
            print_parsing_address_error();
        }
    } else if let Some(file) = args.value_of("file") {
        let socket = args.value_of("socket").unwrap_or("");

        if let Ok(socket_addr) = socket.parse() {
            return AnalyzeFile(file.to_string(), socket_addr);
        } else {
            print_parsing_address_error();
        }
    } else {
        println!("Please provide either an interface or a file. Type --help");
    }
    exit(1)
}

fn print_parsing_address_error() {
    println!("Error parsing address! Expected 168.192.1.3:422 or [2001:db8::1]:8080")
}

fn get_args() -> ArgMatches<'static> {
    App::new("rtpeeker")
        .version("1.0")
        .author("Your Name")
        .about("Your Rust CLI App")
        .subcommand(
            SubCommand::with_name("list")
                .about("List network interfaces")
                .arg(
                    Arg::with_name("up")
                        .short("u")
                        .long("up")
                        .help("Filter devices that are UP"),
                )
                .arg(
                    Arg::with_name("running")
                        .short("r")
                        .long("running")
                        .help("Filter devices that are RUNNING"),
                )
                .arg(
                    Arg::with_name("loopback")
                        .short("l")
                        .long("loopback")
                        .help("Filter devices that are LOOPBACK"),
                )
                .arg(
                    Arg::with_name("wireless")
                        .short("w")
                        .long("wireless")
                        .help("Filter devices that are WIRELESS"),
                ),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run the program")
                .arg(
                    Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .takes_value(true)
                        .value_name("INTERFACE")
                        .help("Network interface name"),
                )
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .long("file")
                        .takes_value(true)
                        .value_name("PATH")
                        .help("Path to the file"),
                )
                .arg(
                    Arg::with_name("socket")
                        .short("s")
                        .long("socket")
                        .takes_value(true)
                        .value_name("SOCKET")
                        .help("Socket addr for example 1.1.1.1:21 or [2001:db8::1]:8080"),
                ),
        )
        .get_matches()
}
