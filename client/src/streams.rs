use packets::Packets;
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::{packet::TransportProtocol, Packet, RtcpPacket};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;
use stream::Stream;

mod packets;
pub mod stream;

pub type RefStreams = Rc<RefCell<Streams>>;
pub type StreamKey = (SocketAddr, SocketAddr, TransportProtocol, u32);

#[derive(Debug, Default)]
pub struct Streams {
    pub packets: Packets,
    pub streams: HashMap<StreamKey, Stream>,
}

impl Streams {
    pub fn clear(&mut self) {
        self.packets.clear();
        self.streams.clear();
    }

    pub fn add_packet(&mut self, packet: Packet) {
        let is_new = self.packets.is_new(&packet);

        if is_new {
            handle_packet(&mut self.streams, &packet);
            self.packets.add_packet(packet);
        } else {
            // if the packet is not new (its id is smaller that the last packet's id)
            // that this must be result of `parse_as` request or refetch (tho packets should be
            // pruned before refetch) in that case, recalculate everything,
            // this can be optimised if it proves to be to slow
            self.packets.add_packet(packet);
            self.recalculate();
        }
    }

    fn recalculate(&mut self) {
        let mut new_streams = HashMap::new();

        self.packets
            .values()
            .for_each(|packet| handle_packet(&mut new_streams, packet));

        self.streams = new_streams;
    }
}

// this function need to take streams as an argument as opposed to methods on `Streams`
// to make `Streams::recalculate` work, dunno if there's a better way
fn handle_packet(streams: &mut HashMap<StreamKey, Stream>, packet: &Packet) {
    match packet.contents {
        SessionPacket::Rtp(ref pack) => {
            let stream = get_or_create_stream(
                streams,
                packet.source_addr,
                packet.destination_addr,
                packet.transport_protocol,
                pack.ssrc,
            );
            stream.add_rtp_packet(packet.id, packet.timestamp, pack);
        }
        SessionPacket::Rtcp(ref packs) => {
            for pack in packs {
                let ssrcs = match pack {
                    RtcpPacket::SenderReport(sr) => vec![sr.ssrc],
                    RtcpPacket::ReceiverReport(rr) => vec![rr.ssrc],
                    RtcpPacket::SourceDescription(sd) => {
                        sd.chunks.iter().map(|chunk| chunk.source).collect()
                    }
                    // TODO DAMIAN what about others?

                    _ => Vec::new(),
                };

                // RtcpPacket::SenderReport(sender_report) => {
                //     if let Some(stream) = streams.get_mut(&sender_report.ssrc) {
                //         stream.add_rtcp_packet(packet)
                //     }
                // }
                // RtcpPacket::ReceiverReport(receiver_report) => {
                //     for report in &receiver_report.reports {
                //         if let Some(stream) = streams.get_mut(&report.ssrc) {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     }
                // }
                // RtcpPacket::SourceDescription(source_description) => {
                //     for chunk in &source_description.chunks {
                //         if let Some(stream) = streams.get_mut(&chunk.source) {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     }
                // }
                // RtcpPacket::Goodbye(goodbye) => {
                //     for source in &goodbye.sources {
                //         if let Some(stream) = streams.get_mut(source) {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     }
                // }
                // RtcpPacket::ApplicationDefined(_) => {
                //     streams.iter_mut().for_each(|(_, stream)| {
                //         if stream.source_addr == packet.source_addr
                //             && stream.destination_addr == packet.destination_addr
                //         {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     });
                // }
                // RtcpPacket::PayloadSpecificFeedback(_) => {
                //     streams.iter_mut().for_each(|(_, stream)| {
                //         if stream.source_addr == packet.source_addr
                //             && stream.destination_addr == packet.destination_addr
                //         {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     });
                // }
                // RtcpPacket::TransportSpecificFeedback(_) => {
                //     streams.iter_mut().for_each(|(_, stream)| {
                //         if stream.source_addr == packet.source_addr
                //             && stream.destination_addr == packet.destination_addr
                //         {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     });
                // }
                // RtcpPacket::ExtendedReport(_) => {
                //     streams.iter_mut().for_each(|(_, stream)| {
                //         if stream.source_addr == packet.source_addr
                //             && stream.destination_addr == packet.destination_addr
                //         {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     });
                // }
                // RtcpPacket::Other(_) => {
                //     streams.iter_mut().for_each(|(_, stream)| {
                //         if stream.source_addr == packet.source_addr
                //             && stream.destination_addr == packet.destination_addr
                //         {
                //             stream.add_rtcp_packet(packet)
                //         }
                //     });
                // }


                for ssrc in ssrcs {
                    let maybe_stream = get_rtcp_stream(
                        streams,
                        packet.source_addr,
                        packet.destination_addr,
                        packet.transport_protocol,
                        ssrc,
                    );
                    if let Some(stream) = maybe_stream {
                        stream.add_rtcp_packet(packet.id, packet.timestamp, pack);
                    }
                }
            }
        }
        // Ignoring other types of RTCP packet for now
        _ => {}
    };
}

fn get_or_create_stream(
    streams: &mut HashMap<StreamKey, Stream>,
    source_addr: SocketAddr,
    destination_addr: SocketAddr,
    protocol: TransportProtocol,
    ssrc: u32,
) -> &mut Stream {
    let streams_len = streams.len();
    let stream_key = (source_addr, destination_addr, protocol, ssrc);
    streams.entry(stream_key).or_insert_with(|| {
        Stream::new(
            source_addr,
            destination_addr,
            protocol,
            ssrc,
            int_to_letter(streams_len),
        )
    })
}

fn get_rtcp_stream(
    streams: &mut HashMap<StreamKey, Stream>,
    mut source_addr: SocketAddr,
    mut destination_addr: SocketAddr,
    protocol: TransportProtocol,
    ssrc: u32,
) -> Option<&mut Stream> {
    let key_same_port = (source_addr, destination_addr, protocol, ssrc);
    if streams.contains_key(&key_same_port) {
        streams.get_mut(&key_same_port)
    } else {
        source_addr.set_port(source_addr.port() + 1);
        destination_addr.set_port(destination_addr.port() + 1);
        let key_next_port = (source_addr, destination_addr, protocol, ssrc);
        streams.get_mut(&key_next_port)
    }
}

fn int_to_letter(unique_id: usize) -> String {
    if unique_id == 0 {
        return String::from("A");
    }
    let mut result = String::new();
    let mut remaining = unique_id;

    while remaining > 0 {
        let current = (remaining) % 26;
        result.insert(0, (b'A' + current as u8) as char);
        remaining /= 26;
    }

    result
}
