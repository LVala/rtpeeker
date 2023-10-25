use crate::server;
use crate::sniffer::Sniffer;
use clap::Args;
use pcap::{Active, Offline};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const DEFAULT_PORT: u16 = 3550;
const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, Args)]
pub struct Run {
    /// Pcap files to capture the packets from
    #[arg(short, long, num_args = 1..)]
    files: Vec<String>,
    /// Network interfaces to capture the packets from
    #[arg(short, long, num_args = 1..)]
    interfaces: Vec<String>,
    /// IP address used by the application
    #[arg(short, long, default_value_t = DEFAULT_IP)]
    address: IpAddr,
    /// Port used by the application
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

impl Run {
    pub async fn run(self) {
        if self.files.is_empty() && self.interfaces.is_empty() {
            // TODO: use some pretty printing (colors, bold font etc.)
            println!("Warning: no pcap files or network interfaces were passed");
        }

        let file_sniffers = get_offline_sniffers(self.files);
        let interface_sniffers = get_online_sniffers(self.interfaces);
        let address = SocketAddr::new(self.address, self.port);

        server::run(interface_sniffers, file_sniffers, address).await;
    }
}

// TODO: refactor, these functions are very similar
fn get_offline_sniffers(files: Vec<String>) -> HashMap<String, Sniffer<Offline>> {
    files.sort_unstable();
    files.dedup();
    files
        .into_iter()
        .filter_map(|file| match Sniffer::from_file(&file) {
            Ok(sniffer) => Some((file, sniffer)),
            Err(err) => {
                println!("Couldn't capture from file {}", file);
                None
            }
        })
        .collect()
}

fn get_online_sniffers(interfaces: Vec<String>) -> HashMap<String, Sniffer<Active>> {
    interfaces.sort_unstable();
    interfaces.dedup();
    interfaces
        .into_iter()
        .filter_map(|interface| match Sniffer::from_device(&interface) {
            Ok(sniffer) => Some((interface, sniffer)),
            Err(err) => {
                println!("Couldn't capture from interface {}", interface);
                None
            }
        })
        .collect()
}
