mod server;
mod sniffer;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let _sniffer = sniffer::Sniffer::from_device("lo");
    let sniffer = sniffer::Sniffer::from_file("./pcap_examples/rtp.pcap").unwrap();

    server::run(sniffer, "127.0.0.1:3550".parse().unwrap()).await;
}
