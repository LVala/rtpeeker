use crate::server;
use crate::sniffer::Sniffer;
use clap::Args;
use std::net::SocketAddr;

static DEFAULT_PORT: &str = "3550";
static DEFAULT_IP: &str = "0.0.0.0";

#[derive(Debug, Args)]
pub struct Run {
    /// Interface name, if file flag, then it is path to pcap file
    input: String,
    /// File path
    #[arg(short, long, default_value_t = false)]
    file: bool,
    /// ip address, if not specified, 0.0.0.0 is used
    #[arg(short, long)]
    address: Option<String>,
    /// port, if not specified, 3550 is used
    #[arg(short, long)]
    port: Option<String>,
}

impl Run {
    pub async fn run(self) {
        let ip = self.address.unwrap_or(DEFAULT_IP.to_string());
        let port = self.port.unwrap_or(DEFAULT_PORT.to_string());
        let address = format!("{ip}:{port}");

        let Ok(address) = address.parse() else {
            println!(
                "Error: IP address or port are invalid"
            );
            return;
        };

        if self.file {
            analyze_file(self.input, address).await
        } else {
            capture_packets(self.input, address).await
        }
    }
}

async fn analyze_file(file: String, address: SocketAddr) {
    let Ok(sniffer) = Sniffer::from_file(&file) else {
        println!("Error:cannot open network interface");
        return;
    };

    server::run(sniffer, address).await;
}

async fn capture_packets(interface: String, address: SocketAddr) {
    let Ok(sniffer) = Sniffer::from_device(&interface) else {
        println!("Error: cannot open network interface");
        return;
    };

    server::run(sniffer, address).await;
}
