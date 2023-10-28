use crate::server;
use crate::sniffer::{Error, Sniffer};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const DEFAULT_PORT: u16 = 3550;
const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, clap::Args)]
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
            println!("Error: no pcap files or network interfaces were passed");
            return;
        }

        let file_sniffers = get_sniffers(self.files, Sniffer::from_file);
        let interface_sniffers = get_sniffers(self.interfaces, Sniffer::from_device);
        let address = SocketAddr::new(self.address, self.port);

        server::run(interface_sniffers, file_sniffers, address).await;
    }
}

fn get_sniffers<T, F>(mut sources: Vec<String>, get_sniffer: F) -> HashMap<String, Sniffer<T>>
where
    T: pcap::Activated,
    F: Fn(&str) -> Result<Sniffer<T>, Error>,
{
    sources.sort_unstable();
    sources.dedup();
    sources
        .into_iter()
        .filter_map(|source| match get_sniffer(&source) {
            Ok(sniffer) => Some((source, sniffer)),
            Err(_) => {
                println!("Failed to capture packets from source {}", source);
                None
            }
        })
        .collect()
}
