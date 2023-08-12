use std::io;
use pcap::{Device, ConnectionStatus, IfFlags};
use std::net::IpAddr;



// #[tokio::main]
// async fn main() {
//     warp::serve(warp::fs::dir("client/dist"))
//         .run(([127, 0, 0, 1], 3550))
//         .await;
// }

use rtpeeker::sniffer;

#[tokio::main]
async fn main() {
    let devices = Device::list().expect("Error listing devices");

    fn main() -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1600.0, 640.0)),
            ..Default::default()
        };
        eframe::run_native(
            "Media Stream Analyzer",
            options,
            Box::new(|_cc| Box::new(ViewState::new())),
        )
    }

    list_devices(devices);
    let device = select_device();
    // TODO first choose file (open dialog)
    // TODO loop on error



    let Ok(mut sniffer) = sniffer::Sniffer::from_device(device.as_str()) else {
        println!("Cannot open file");
        return;
    };

    while let Ok(mut packet) = sniffer.next_packet() {
        packet.parse_as(sniffer::packet::PacketType::RtpOverUdp);
        println!("{:?}", packet);
    }


    // let Ok(mut sniffer) = sniffer::Sniffer::from_file("./pcap_examples/rtp.pcap") else {
    //     println!("Cannot open file");
    //     return;
    // };
    //
}


fn format_flags(flags: IfFlags) -> String {
    let mut result = Vec::new();

    if flags.contains(IfFlags::LOOPBACK) {
        result.push("LOOPBACK");
    }
    if flags.contains(IfFlags::UP) {
        result.push("UP");
    }
    if flags.contains(IfFlags::RUNNING) {
        result.push("RUNNING");
    }
    if flags.contains(IfFlags::WIRELESS) {
        result.push("WIRELESS");
    }

    if result.is_empty() {
        result.push("N/A")
    }
    result.join(" | ")
}

fn format_connection_status(status: ConnectionStatus) -> String {
    match status {
        ConnectionStatus::Unknown => "Unknown".to_string(),
        ConnectionStatus::Connected => "Connected".to_string(),
        ConnectionStatus::Disconnected => "Disconnected".to_string(),
        ConnectionStatus::NotApplicable => "N/A".to_string(),
    }
}

fn format_ip_addr(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4) => ipv4.to_string(),
        IpAddr::V6(ipv6) => ipv6.to_string(),
    }
}

fn format_optional_ip(ip: &Option<IpAddr>) -> String {
    ip.as_ref().map_or("None".to_string(), |addr| format_ip_addr(addr))
}

fn select_device() -> String {
    println!("Enter the name of the chosen device:");
    let mut chosen_name = String::new();
    io::stdin().read_line(&mut chosen_name).expect("Failed to read line");
    chosen_name.trim().to_string()
}

fn list_devices(devices: Vec<Device>) {
    println!("Available network devices:");
    for device in devices {
        println!("Name: {}", device.name);
        println!("Description: {}", device.desc.unwrap_or("N/A".to_string()));
        println!("Addresses: {}", if device.addresses.is_empty() { "N/A" } else { "" });
        for address in &device.addresses {
            println!("  Address: {}", format_ip_addr(&address.addr));
            println!("  Netmask: {}", format_optional_ip(&address.netmask));
            println!("  Broadcast: {}", format_optional_ip(&address.broadcast_addr));
            println!("  Destination: {}\n", format_optional_ip(&address.dst_addr));
        }

        println!("Flags: {}", format_flags(device.flags.if_flags));
        println!("Connection status: {}", format_connection_status(device.flags.connection_status));
        println!("\n");
    }
}
