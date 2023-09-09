use serde::{Deserialize, Serialize};
use MediaType::{Audio, AudioOrVideo, Video};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub csrc: Vec<u32>,
    pub payload_length: usize, // extension information skipped
}

#[cfg(not(target_arch = "wasm32"))]
impl RtpPacket {
    pub fn build(packet: &super::Packet) -> Option<Self> {
        use rtp::packet::Packet;
        use webrtc_util::marshal::Unmarshal;

        // payload field should never be empty
        // except for when encoding the packet
        let mut buffer: &[u8] = packet
            .payload
            .as_ref()
            .expect("Packet's payload field is empty");
        let Ok(Packet { header, payload }) = Packet::unmarshal(&mut buffer) else {
            return None;
        };

        Some(Self {
            version: header.version,
            padding: header.padding,
            extension: header.extension,
            marker: header.marker,
            payload_type: header.payload_type,
            sequence_number: header.sequence_number,
            timestamp: header.timestamp,
            ssrc: header.ssrc,
            csrc: header.csrc,
            payload_length: payload.len(),
        })
    }
}

#[derive(Debug)]
pub enum MediaType {
    Audio,
    Video,
    AudioOrVideo,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_payload_type_info(payload_type: u8) -> (String, MediaType, i32) {
    let (name, media_type, clock_rate_in_hz) = match payload_type {
        0 => ("PCMU", Audio, 8000),
        1 => ("reserved (previously FS-1016 CELP)", Audio, 8000),
        2 => ("reserved (previously FS-1016 CELP)", Audio, 8000),
        3 => ("GSM", Audio, 8000),
        4 => ("G723", Audio, 8000),
        5 => ("DVI4", Audio, 8000),
        6 => ("DVI4", Audio, 16_000),
        7 => ("LPC", Audio, 8000),
        8 => ("PCMA", Audio, 8000),
        9 => ("G722", Audio, 8000),
        10 => ("L16", Audio, 44_100),
        11 => ("L16", Audio, 44_100),
        12 => ("QCELP", Audio, 8000),
        13 => ("CN", Audio, 8000),
        14 => ("MPA", Audio, 90_000),
        15 => ("G728", Audio, 8000),
        16 => ("DVI4", Audio, 11_025),
        17 => ("DVI4", Audio, 22_050),
        18 => ("G729", Audio, 8000),
        19 => ("reserved (previously CN)", Audio, 0),
        25 => ("CELLB", Video, 90_000),
        26 => ("JPEG", Video, 90_000),
        28 => ("nv", Video, 90_000),
        31 => ("H261", Video, 90_000),
        32 => ("MPV", Video, 90_000),
        33 => ("MP2T", AudioOrVideo, 90_000),
        34 => ("H263", Video, 90_000),
        72..=76 => ("reserved", AudioOrVideo, 0),
        77..=79 => ("unassigned", AudioOrVideo, 0),
        _ => ("dynamic", AudioOrVideo, 0),
    };
    (name.to_string(), media_type, clock_rate_in_hz)
}
