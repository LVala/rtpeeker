use crate::sniffer;
use clap::Args;
use sniffer::Sniffer;
use std::net::SocketAddr;
use std::ops::Add;

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
    /// port, if not specified, 8080 is used
    #[arg(short, long)]
    port: Option<String>,
}

impl Run {
    pub async fn run(self) -> Result<(), ()> {
        if self.file {
            self.analyze_file()
                .await
                .expect("analyze file failed");
        } else {
            self.capture_packets()
                .await
                .expect("capture packets failed");
        }
        Ok(())
    }

    async fn analyze_file(self) -> Result<(), ()> {
        let Ok(mut sniffer) = Sniffer::from_file(self.input.as_str()) else {
            println!("Cannot open file");
            return Err(());
        };

        while let Ok(mut packet) = sniffer.next_packet() {
            packet.parse_as(sniffer::packet::PacketType::RtpOverUdp);
            println!("{:?}", packet);
        }
        self.warp_serve().await
    }

    async fn capture_packets(self) -> Result<(), ()> {
        let Ok(mut sniffer) = Sniffer::from_device(self.input.as_str()) else {
            println!("Cannot open network interface");
            return Err(());
        };

        while let Ok(mut packet) = sniffer.next_packet() {
            packet.parse_as(sniffer::packet::PacketType::RtpOverUdp);
            println!("{:?}", packet);
        }
        self.warp_serve().await
    }

    async fn warp_serve(self) -> Result<(), ()> {
        let address = self.address.unwrap_or("0.0.0.0".to_string());
        let port = self.port.unwrap_or("8080".to_string());
        let combined = address.add(port.as_str());
        if let Ok(socket_addr) = combined.parse() {
            let socket: SocketAddr = socket_addr;
            warp::serve(warp::fs::dir("client/dist")).run(socket).await;
            Ok(())
        } else {
            println!(
                "Parsing socket address failed. Expected 168.192.1.3:422 or [2001:db8::1]:8080 "
            );
            Err(())
        }
    }
}
