use std::io;
use pcap::{Device, ConnectionStatus, IfFlags};
use std::net::IpAddr;


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

#[tokio::main]
async fn main() {
    let devices = Device::list().expect("Error listing devices");

    listDevices(devices);
    selectDevice();
    // TODO first choose file (open dialog)
    // TODO loop on error

    // warp::serve(warp::fs::dir("client/dist"))
    //     .run(([127, 0, 0, 1], 3550))
    //     .await;
}

fn selectDevice() {
    println!("Enter the name of the chosen device:");
    let mut chosen_name = String::new();
    io::stdin().read_line(&mut chosen_name).expect("Failed to read line");
    let chosen_name = chosen_name.trim(); // Remove newline characters
    println!("You chose: {}", chosen_name);
}

fn listDevices(devices: Vec<Device>) {
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
