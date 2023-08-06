// #[tokio::main]
// async fn main() {
//     warp::serve(warp::fs::dir("client/dist"))
//         .run(([127, 0, 0, 1], 3550))
//         .await;
// }

use rtpeeker::sniffer;

fn main() {
    let Ok(mut sniffer) = sniffer::Sniffer::from_file("./pcap_examples/rtp.pcap") else {
        println!("Cannot open file");
        return;
    };

    while let Ok(mut packet) = sniffer.next_packet() {
        packet.parse_as(sniffer::packet::PacketType::RtpOverUdp);
        println!("{:?}", packet);
    }
}
