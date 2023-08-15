use pcap::{ConnectionStatus, Device, IfFlags};
use std::net::IpAddr;

pub(crate) fn list_devices(flags: Vec<String>) {
    println!("Available network devices:");
    let devices = Device::list().expect("Error listing devices");
    for device in devices {
        let formatted_flags = format_flags(device.flags.if_flags);
        let mut should_print = true;
        for flag_name in &flags {
            if !formatted_flags.contains(flag_name) {
                should_print = false;
            }
        }
        if formatted_flags.eq("None") && !flags.is_empty() {
            should_print = false
        }
        if !should_print {
            continue;
        }
        println!("Name: {}", device.name);
        println!("Description: {}", device.desc.unwrap_or("None".to_string()));
        println!(
            "Addresses: {}",
            if device.addresses.is_empty() {
                "None"
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
        println!("Flags: {}", formatted_flags);
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
        result.push("None")
    }
    result.join(" | ")
}

fn format_connection_status(status: ConnectionStatus) -> String {
    match status {
        ConnectionStatus::Unknown => "Unknown".to_string(),
        ConnectionStatus::Connected => "Connected".to_string(),
        ConnectionStatus::Disconnected => "Disconnected".to_string(),
        ConnectionStatus::NotApplicable => "None".to_string(),
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
