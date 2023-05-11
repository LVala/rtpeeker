use std::path::Path;

pub mod rtp_sniffer;


fn main() {
    let file = Path::new("./pcap_examples/sip_rtp.pcap");
    rtp_sniffer::rtp_from_file(file);
}
