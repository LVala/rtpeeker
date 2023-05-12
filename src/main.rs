use std::path::Path;

pub mod rtp_sniffer;

fn main() {
    let file = Path::new("./pcap_examples/rtp.pcap");
    let packets = rtp_sniffer::rtp_from_file(file);
    for packet in packets {
        println!("{:?}", packet);
    }
}
