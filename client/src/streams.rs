use packets::Packets;
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::{Packet, RtpPacket};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use stream::Stream;

mod packets;
mod stream;

pub type RefStreams = Rc<RefCell<Streams>>;

#[derive(Debug, Default)]
pub struct Streams {
    pub packets: Packets,
    pub streams: HashMap<u32, Stream>,
}

impl Streams {
    pub fn clear_all_packets(&mut self) {
        self.packets.clear();
        self.streams.clear();
    }

    pub fn add_packet(&mut self, mut packet: Packet) {
        let is_new = self.packets.is_new(&packet);

        if is_new {
            self.detect_previous_packet_lost(&mut packet);
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

    fn detect_previous_packet_lost(&mut self, packet: &mut Packet) {
        if let SessionPacket::Rtp(ref mut new_rtp) = packet.contents {
            self.update_subsequent_packet(new_rtp);
            if !self.previous_packet_present(new_rtp) {
                new_rtp.previous_packet_is_lost = true
            }
        };
    }

    fn update_subsequent_packet(&mut self, new_rtp: &mut RtpPacket) {
        if let Some(stream) = self.streams.get_mut(&new_rtp.ssrc) {
            stream
                .rtp_packets
                .iter()
                .rev()
                .take(10)
                .for_each(|rtp_pack_id| {
                    let rtp_packet = self.packets.get_mut(*rtp_pack_id).unwrap();
                    let SessionPacket::Rtp(ref mut rtp) = rtp_packet.contents else {
                        unreachable!();
                    };

                    if rtp.sequence_number == new_rtp.sequence_number + 1 {
                        rtp.previous_packet_is_lost = false
                    }
                });
        }
    }

    fn previous_packet_present(&mut self, new_rtp: &mut RtpPacket) -> bool {
        if let Some(stream) = self.streams.get(&new_rtp.ssrc) {
            stream.rtp_packets.iter().rev().take(10).any(|rtp_pack_id| {
                let rtp_packet = self.packets.get(*rtp_pack_id).unwrap();
                let SessionPacket::Rtp(ref rtp) = rtp_packet.contents else {
                    unreachable!();
                };

                rtp.sequence_number == new_rtp.sequence_number - 1
            })
        } else {
            // stream not present - it is first packet
            true
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
fn handle_packet(streams: &mut HashMap<u32, Stream>, packet: &Packet) {
    match packet.contents {
        SessionPacket::Rtp(ref pack) => {
            let streams_len = streams.len();
            streams.entry(pack.ssrc).or_insert_with(|| {
                Stream::new(
                    packet.source_addr,
                    packet.destination_addr,
                    pack.ssrc,
                    pack.payload_type.id,
                    int_to_letter(streams_len),
                )
            });
            streams
                .get_mut(&pack.ssrc)
                .unwrap()
                .add_rtp_packet(packet, pack);
        }
        SessionPacket::Rtcp(ref _packs) => {
            // TODO: handle RTCP packets
        }
        _ => {}
    };
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

pub fn is_stream_visible(streams_visibility: &mut HashMap<u32, bool>, ssrc: u32) -> &mut bool {
    streams_visibility.entry(ssrc).or_insert(true)
}
