use rtpeeker_common::packet::Packet;
use std::collections::{
    btree_map::{Keys, Values},
    BTreeMap,
};

#[derive(Debug, Default)]
pub struct Packets {
    packets: BTreeMap<usize, Packet>,
}

impl Packets {
    pub fn get(&self, id: usize) -> Option<&Packet> {
        self.packets.get(&id)
    }

    pub fn first(&self) -> Option<&Packet> {
        match self.packets.first_key_value() {
            Some((_, v)) => Some(v),
            _ => None,
        }
    }

    pub fn values(&self) -> Values<'_, usize, Packet> {
        self.packets.values()
    }

    pub fn keys(&self) -> Keys<'_, usize, Packet> {
        self.packets.keys()
    }

    pub fn is_new(&self, packet: &Packet) -> bool {
        !self.packets.contains_key(&packet.id)
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    pub fn len(&self) -> usize {
        self.packets.len()
    }

    pub fn clear(&mut self) {
        self.packets.clear();
    }

    pub fn id_count(&self) -> usize {
        match self.packets.last_key_value() {
            Some((id, _)) => *id,
            None => 0,
        }
    }

    pub fn add_packet(&mut self, packet: Packet) {
        self.packets.insert(packet.id, packet);
    }
}
