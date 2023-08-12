use rtpeeker::command_line_interface::start_interface::command_line_interface;
use rtpeeker::command_line_interface::start_interface::Action::{AnalyzeFile, CapturePackets};
use rtpeeker::sniffer;

// #[tokio::main]
// async fn main() {
//     warp::serve(warp::fs::dir("client/dist"))
//         .run(([127, 0, 0, 1], 3550))
//         .await;
// }

#[tokio::main]
async fn main() {
    let action = command_line_interface();
    match action {
        CapturePackets(device) => capture_packets(device),
        AnalyzeFile(path) => analyze_file(path),
    };
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
