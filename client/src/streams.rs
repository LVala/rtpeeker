use packets::Packets;
use rtpeeker_common::packet::SessionPacket;
use rtpeeker_common::Packet;
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
        remaining = (remaining) / 26;
    }

    result
}
