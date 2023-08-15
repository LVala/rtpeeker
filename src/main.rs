use std::net::SocketAddr;

use rtpeeker::command_line_interface::start_interface::start_command_line_interface;
use rtpeeker::command_line_interface::start_interface::Action::{AnalyzeFile, CapturePackets};
use rtpeeker::sniffer;

#[tokio::main]
async fn main() {
    let action = start_command_line_interface();
    match action {
        CapturePackets(device, socket_addr) => {
            capture_packets(device);
            warp_serve(socket_addr).await
        }
        AnalyzeFile(path, socket_addr) => {
            analyze_file(path);
            warp_serve(socket_addr).await
        }
    };
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
