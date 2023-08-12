use pcap::{ConnectionStatus, Device, IfFlags};
use std::io::stdin;
use std::net::IpAddr;
use std::thread::sleep;
use std::time::Duration;

pub(crate) fn select_device() -> String {
    loop {
        list_devices();
        println!("Enter the name of the chosen device:");
        let mut chosen_name = String::new();
        stdin()
            .read_line(&mut chosen_name)
            .expect("Failed to read line");
        let chosen_name = chosen_name.trim().to_string();

        if Device::list()
            .expect("Error listing devices")
            .iter()
            .any(|dev| dev.name == chosen_name)
        {
            return chosen_name;
        } else {
            println!("\nError: Invalid device name. Please choose a valid device.\n");
            sleep(Duration::new(1, 0))
        }
    }
}

fn list_devices() {
    println!("Available network devices:");
    let devices = Device::list().expect("Error listing devices");
    for device in devices {
        println!("Name: {}", device.name);
        println!("Description: {}", device.desc.unwrap_or("N/A".to_string()));
        println!(
            "Addresses: {}",
            if device.addresses.is_empty() {
                "N/A"
            } else {
                ""
            }
        );
        for address in &device.addresses {
            println!("  Address: {}", format_ip_addr(&address.addr));
            println!("  Netmask: {}", format_optional_ip(&address.netmask));
            println!(
                "  Broadcast: {}",
                format_optional_ip(&address.broadcast_addr)
            );
            println!("  Destination: {}\n", format_optional_ip(&address.dst_addr));
        }

        println!("Flags: {}", format_flags(device.flags.if_flags));
        println!(
            "Connection status: {}",
            format_connection_status(device.flags.connection_status)
        );
        println!("--------------------------------------\n");
    }
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
    ip.as_ref()
        .map_or("None".to_string(), |addr| format_ip_addr(addr))
}
