use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

// use rtpeeker::command_line_interface::start_interface::command_line_interface;
// use rtpeeker::command_line_interface::start_interface::Action::{AnalyzeFile, CapturePackets};
use rtpeeker::{cmd, sniffer};

#[tokio::main]
async fn main() {
    let x = RtpeekerArgs::parse();
    println!("{:?}", x);
    // let action = command_line_interface();
    // match action {
    //     CapturePackets(device, socket_addr) => {
    //         capture_packets(device);
    //         warp_serve(socket_addr).await
    //     }
    //     AnalyzeFile(path, socket_addr) => {
    //         analyze_file(path);
    //         warp_serve(socket_addr).await
    //     }
    // };
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct RtpeekerArgs {
    #[clap(subcommand)]
    action: CommandType,
}

#[derive(Debug, Subcommand)]
enum CommandType {
    File(FileSubcommand),
    Interface(InterfaceSubcommand),
    ListInterfaces(ListInterfaceSubcommand),
}

#[derive(Debug, Args)]
struct FileSubcommand {
    file_path: String,
    socket_to_serve: String,
}

#[derive(Debug, Args)]
struct InterfaceSubcommand {
    interface_name: String,
    socket_to_serve: String,
}

#[derive(Debug, Args)]
struct ListInterfaceSubcommand {
    filter_up: Option<bool>,
    filter_loopback: Option<bool>,
    filter_running: Option<bool>,
    filter_wireless: Option<bool>,
}

async fn warp_serve(socket_addr: SocketAddr) {
    warp::serve(warp::fs::dir("client/dist"))
        .run(socket_addr)
        .await
}

fn capture_packets(device: String) {
    let Ok(mut sniffer) = sniffer::Sniffer::from_device(device.as_str()) else {
        println!("Cannot open network interface");
        return;
    };

    while let Ok(mut packet) = sniffer.next_packet() {
        packet.parse_as(sniffer::packet::PacketType::RtpOverUdp);
        println!("{:?}", packet);
    }
}

fn analyze_file(path: String) {
    let Ok(mut sniffer) = sniffer::Sniffer::from_file(path.as_str()) else {
        println!("Cannot open file");
        return;
    };

    while let Ok(mut packet) = sniffer.next_packet() {
        packet.parse_as(sniffer::packet::PacketType::RtpOverUdp);
        println!("{:?}", packet);
    }
}
