use clap::Args;
use pcap::{ConnectionStatus, Device, IfFlags};
use std::net::IpAddr;
use warp::redirect::permanent;

#[derive(Debug, Args)]
pub struct List {}

impl List {
    pub async fn run(self) -> Result<(), ()> {
        list_devices();
        Ok(())
    }
}

fn list_devices() {
    let devices = Device::list().expect("Error listing devices");

    for (ix, device) in devices.iter().enumerate() {
        let formatted_flags = format_flags(device.flags.if_flags);

        if !formatted_flags.contains("up") || device.addresses.is_empty() {
            continue;
        }

        println!("{}. {} ({})", ix, device.name, formatted_flags);
        println!("Addrs:");
        for address in &device.addresses {
            println!(
                "  {} (mask {})",
                format_ip_addr(&address.addr),
                format_optional_ip(&address.netmask)
            );

            if let Some(broadcast) = address.broadcast_addr {
                println!("  {} (broadcast)", broadcast.to_string());
            }
            if let Some(destination) = address.dst_addr {
                println!("  {} (destination)", destination.to_string());
            }
        }
        println!()
    }
}

fn format_flags(flags: IfFlags) -> String {
    let mut result = Vec::new();

    if flags.contains(IfFlags::LOOPBACK) {
        result.push("loopback");
    }
    if flags.contains(IfFlags::UP) {
        result.push("up");
    }
    if flags.contains(IfFlags::RUNNING) {
        result.push("running");
    }
    if flags.contains(IfFlags::WIRELESS) {
        result.push("wireless");
    }

    if result.is_empty() {
        result.push("None")
    }
    result.join(" | ")
}

fn format_ip_addr(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4) => ipv4.to_string(),
        IpAddr::V6(ipv6) => ipv6.to_string(),
    }
}

fn format_optional_ip(ip: &Option<IpAddr>) -> String {
    ip.as_ref().map_or("None".to_string(), format_ip_addr)
}
