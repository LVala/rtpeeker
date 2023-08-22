use clap::Args;
use pcap::{Device, IfFlags};
use std::net::IpAddr;

#[derive(Debug, Args)]
pub struct List {}

impl List {
    pub async fn run(self) {
        Device::list()
            .expect("Error occured while listing devices")
            .iter()
            .filter(|device| {
                (device.flags.if_flags.contains(IfFlags::UP) && !device.addresses.is_empty())
                    || device.name == "any"
            })
            .enumerate()
            .for_each(|(ix, device)| {
                let flags = format_flags(device.flags.if_flags);
                println!("{}. {} ({})", ix, device.name, flags);
                if !device.addresses.is_empty() {
                    println!("Addrs:");
                }
                for address in &device.addresses {
                    println!(
                        "  {} (mask {})",
                        format_ip_addr(&address.addr),
                        format_optional_ip(&address.netmask)
                    );
                }
                println!();
            });
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
