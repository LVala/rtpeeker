use crate::server;
use crate::sniffer::Sniffer;
use clap::Args;
use pcap::{Active, Offline};
use std::collections::HashMap;
use std::fs;

const DEFAULT_PORT: &str = "3550";
const DEFAULT_IP: &str = "0.0.0.0";

#[derive(Debug, Args)]
pub struct Run {
    /// Network interface name or file path
    input: String,
    /// If set, input argument will be treated as a file path, not interface name
    #[arg(short, long, default_value_t = false)]
    file: bool,
    /// IP address, if not specified, 0.0.0.0 is used
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
            println!("Error: IP address or port are invalid");
            return;
        };

        let file_sniffers: HashMap<String, Sniffer<Offline>> = get_offline_sniffers();
        let interface_sniffers: HashMap<String, Sniffer<Active>> =
            get_online_sniffers(self.input.clone());

        server::run(
            self.input.clone(),
            interface_sniffers,
            file_sniffers,
            address,
        )
        .await;
    }
}

fn get_offline_sniffers() -> HashMap<String, Sniffer<Offline>> {
    let mut hash_map = HashMap::new();
    fs::read_dir("pcap_examples")
        .unwrap()
        .map(|path| String::from(path.unwrap().file_name().to_str().unwrap()))
        .for_each(|filename| {
            let file_path = format!("pcap_examples/{}", filename);

            let Ok(sniffer) = Sniffer::from_file(file_path.as_str(), filename.as_str()) else {
                println!("Error:cannot open the file");
                return;
            };

            hash_map.insert(filename, sniffer);
        });

    hash_map
}

fn get_online_sniffers(device: String) -> HashMap<String, Sniffer<Active>> {
    let mut hash_map = HashMap::new();

    let Ok(sniffer) = Sniffer::from_device(device.as_str()) else {
        println!("Error:cannot open the file");
        return hash_map;
    };
    hash_map.insert(device, sniffer);

    hash_map
}
