pub mod adaptation_field;
pub mod aggregator;
pub mod descriptors;
pub mod header;
pub mod payload;
pub mod pes;
pub mod psi;

use crate::mpegts::adaptation_field::AdaptationField;
use crate::mpegts::header::Header;
#[cfg(not(target_arch = "wasm32"))]
use crate::mpegts::header::{AdaptationFieldControl, PIDTable, TransportScramblingControl};
use crate::mpegts::payload::RawPayload;
use serde::{Deserialize, Serialize};

pub const PAYLOAD_LENGTH: usize = 1316;
pub const FRAGMENT_SIZE: usize = 188;
pub const HEADER_SIZE: usize = 4;
pub const SYNC_BYTE: u8 = 0x47;
pub const SYNC_BYTE_MASK: u8 = 0xFF;
pub const TEI_MASK: u8 = 0x80;
pub const PUSI_MASK: u8 = 0x40;
pub const TP_MASK: u8 = 0x20;
pub const PID_MASK_UPPER: u8 = 0x1F;
pub const TSC_MASK: u8 = 0xC0;
pub const AFC_MASK: u8 = 0x30;
pub const CC_MASK: u8 = 0x0F;
pub const PADDING_BYTE: u8 = 0xFF;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MpegtsPacket {
    pub number_of_fragments: usize,
    pub fragments: Vec<MpegtsFragment>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MpegtsFragment {
    pub header: Header,
    pub adaptation_field: Option<AdaptationField>,
    pub payload: Option<RawPayload>,
}

#[cfg(not(target_arch = "wasm32"))]
impl MpegtsPacket {
    pub fn build(packet: &super::Packet) -> Option<Self> {
        let Some(payload) = packet.payload.as_ref() else {
            return None;
        };
        let Some(packet) = Self::unmarshall(payload) else {
            return None;
        };
        Some(packet)
    }

    fn unmarshall(buffer: &Vec<u8>) -> Option<Self> {
        if buffer.len() != PAYLOAD_LENGTH {
            return None;
        }
        let mut number_of_fragments: usize = 0;
        let mut fragments: Vec<MpegtsFragment> = vec![];

        while (number_of_fragments * FRAGMENT_SIZE) < PAYLOAD_LENGTH
            && (buffer[number_of_fragments * FRAGMENT_SIZE] & SYNC_BYTE_MASK) == SYNC_BYTE
        {
            let Some(fragment) = Self::get_fragment(
                buffer,
                number_of_fragments * FRAGMENT_SIZE,
                number_of_fragments,
            ) else {
                break;
            };
            fragments.push(fragment);
            number_of_fragments += 1;
        }
        match fragments.len() {
            0 => None,
            _ => Some(Self {
                number_of_fragments,
                fragments,
            }),
        }
    }

    fn get_fragment(
        buffer: &Vec<u8>,
        mut start_index: usize,
        fragment_number: usize,
    ) -> Option<MpegtsFragment> {
        let Some(header) = Self::get_header(buffer, start_index) else {
            return None;
        };
        start_index += HEADER_SIZE;
        let adaptation_field = match header.adaptation_field_control {
            AdaptationFieldControl::AdaptationFieldOnly
            | AdaptationFieldControl::AdaptationFieldAndPaylod => {
                let Some(adaptation_field) = Self::get_adaptation_field(buffer, start_index) else {
                    return None;
                };
                start_index += adaptation_field.adaptation_field_length as usize;
                Some(adaptation_field)
            }
            _ => None,
        };

        let payload = match header.adaptation_field_control {
            AdaptationFieldControl::PayloadOnly
            | AdaptationFieldControl::AdaptationFieldAndPaylod => {
                let Some(payload) = Self::get_payload(buffer, start_index, fragment_number) else {
                    return None;
                };
                Some(payload)
            }
            _ => None,
        };
        Some(MpegtsFragment {
            header,
            adaptation_field,
            payload,
        })
    }

    fn get_header(buffer: &Vec<u8>, start_index: usize) -> Option<Header> {
        let transport_error_indicator = ((buffer[start_index + 1] & TEI_MASK) >> 7) == 1;
        let payload_unit_start_indicator = ((buffer[start_index + 1] & PUSI_MASK) >> 6) == 1;
        let transport_priority = ((buffer[start_index + 1] & TP_MASK) >> 5) == 1;
        let pid = ((buffer[start_index + 1] & PID_MASK_UPPER) as u16) << 8
            | buffer[start_index + 2] as u16;
        let pid: PIDTable = PIDTable::from(pid);

        let transport_scrambling_control = match (buffer[start_index + 3] & TSC_MASK) >> 6 {
            0 => TransportScramblingControl::NotScrambled,
            val => TransportScramblingControl::UserDefined(val),
        };
        let adaptation_field_control = match (buffer[start_index + 3] & AFC_MASK) >> 4 {
            1 => AdaptationFieldControl::PayloadOnly,
            2 => AdaptationFieldControl::AdaptationFieldOnly,
            3 => AdaptationFieldControl::AdaptationFieldAndPaylod,
            _ => return None,
        };
        let continuity_counter = buffer[start_index + 3] & CC_MASK;
        Some(Header {
            transport_error_indicator,
            payload_unit_start_indicator,
            transport_priority,
            pid,
            transport_scrambling_control,
            adaptation_field_control,
            continuity_counter,
        })
    }

    fn get_adaptation_field(buffer: &Vec<u8>, start_index: usize) -> Option<AdaptationField> {
        let adaptation_field_length = buffer[start_index];
        Some(AdaptationField {
            adaptation_field_length,
        })
    }

    fn get_payload(
        buffer: &Vec<u8>,
        start_index: usize,
        fragment_number: usize,
    ) -> Option<RawPayload> {
        let end_index = (fragment_number + 1) * FRAGMENT_SIZE;
        let length = end_index.saturating_sub(start_index);

        if length == 0 {
            return None;
        }

        let data = buffer[start_index..end_index.min(buffer.len())].to_vec();
        Some(RawPayload { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpegts::header::{AdaptationFieldControl, PIDTable, TransportScramblingControl};

    fn create_test_buffer() -> Vec<u8> {
        let mut buffer = vec![0; PAYLOAD_LENGTH];
        for i in 0..7 {
            let start_index = i * FRAGMENT_SIZE;
            buffer[start_index] = SYNC_BYTE;
            buffer[start_index + 1] = 0x40; // Set PUSI
            buffer[start_index + 2] = 0x00; // PID 0x0000
            buffer[start_index + 3] = 0x10; // Payload only, CC = 0
                                            // Fill the rest of the TS packet with dummy payload
            for j in 4..FRAGMENT_SIZE {
                buffer[start_index + j] = (j % 256) as u8;
            }
        }
        buffer
    }

    #[test]
    fn test_unmarshall_valid_packet() {
        let buffer = create_test_buffer();
        let packet = MpegtsPacket::unmarshall(&buffer);
        assert!(packet.is_some(), "Failed to unmarshall packet");
        let packet = packet.unwrap();
        assert_eq!(
            packet.number_of_fragments, 7,
            "Incorrect number of fragments"
        );
        assert_eq!(
            packet.fragments.len(),
            7,
            "Incorrect number of fragments in vec"
        );

        // Test the first fragment
        let first_fragment = &packet.fragments[0];
        assert_eq!(first_fragment.header.pid, PIDTable::ProgramAssociation);
        assert!(first_fragment.header.payload_unit_start_indicator);
        assert!(matches!(
            first_fragment.header.adaptation_field_control,
            AdaptationFieldControl::PayloadOnly
        ));
        assert!(first_fragment.adaptation_field.is_none());
        assert!(first_fragment.payload.is_some());
        assert_eq!(
            first_fragment.payload.as_ref().unwrap().data.len(),
            FRAGMENT_SIZE - HEADER_SIZE
        );
    }

    #[test]
    fn test_unmarshall_invalid_length() {
        let buffer = vec![0; PAYLOAD_LENGTH - 1];
        let packet = MpegtsPacket::unmarshall(&buffer);
        assert!(packet.is_none());
    }

    #[test]
    fn test_get_header() {
        let mut buffer = vec![0; FRAGMENT_SIZE];
        buffer[0] = SYNC_BYTE;
        buffer[1] = 0b01000000; // TEI: 0, PUSI: 1, TP: 0, PID: 0x100 (upper 5 bits)
        buffer[2] = 0b01100100; // PID: 0x100 (lower 8 bits)
        buffer[3] = 0b01010000; // TSC: 01, AFC: 01, CC: 0000

        let header = MpegtsPacket::get_header(&buffer, 0).unwrap();
        assert_eq!(header.transport_error_indicator, false);
        assert_eq!(header.payload_unit_start_indicator, true);
        assert_eq!(header.transport_priority, false);
        assert_eq!(header.pid, PIDTable::PID(0x64));
        assert!(matches!(
            header.transport_scrambling_control,
            TransportScramblingControl::UserDefined(1)
        ));
        assert!(matches!(
            header.adaptation_field_control,
            AdaptationFieldControl::PayloadOnly
        ));
        assert_eq!(header.continuity_counter, 0);
    }

    #[test]
    fn test_get_adaptation_field() {
        let mut buffer = vec![0; FRAGMENT_SIZE];
        buffer[4] = 10; // Adaptation field length

        let adaptation_field = MpegtsPacket::get_adaptation_field(&buffer, 4).unwrap();
        assert_eq!(adaptation_field.adaptation_field_length, 10);
    }

    #[test]
    fn test_get_payload() {
        let mut buffer = vec![0; FRAGMENT_SIZE];
        for i in 4..FRAGMENT_SIZE {
            buffer[i] = i as u8;
        }

        let payload = MpegtsPacket::get_payload(&buffer, 4, 0).unwrap();
        assert_eq!(payload.data.len(), FRAGMENT_SIZE - 4);
        assert_eq!(payload.data[0], 4);
        assert_eq!(
            payload.data[payload.data.len() - 1],
            (FRAGMENT_SIZE - 1) as u8
        );
    }

    #[test]
    fn test_get_fragment() {
        let mut buffer = create_test_buffer();
        buffer[1] = 0b00000000; // PID: 0x000 (upper 5 bits)
        buffer[2] = 0; // PID: 0x000 (lower 8 bits)
        buffer[3] = 0b00010000; // AFC: 01 (payload only)

        let fragment = MpegtsPacket::get_fragment(&buffer, 0, 0).unwrap();
        assert!(matches!(fragment.header.pid, PIDTable::ProgramAssociation));
        assert!(matches!(
            fragment.header.adaptation_field_control,
            AdaptationFieldControl::PayloadOnly
        ));
        assert!(fragment.adaptation_field.is_none());
        assert!(fragment.payload.is_some());
    }
}
