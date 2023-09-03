use crate::app::gui::payload_type::MediaType::{Audio, AudioOrVideo, Video};

#[derive(Debug)]
pub struct PayloadType {
    pub id: u8,
    pub name: String,
    pub clock_rate_in_hz: Option<u32>,
    pub media_type: MediaType,
}

#[derive(Debug)]
pub enum MediaType {
    Audio,
    Video,
    AudioOrVideo,
}

impl PayloadType {
    pub fn new(payload_type: u8) -> Self {
        match payload_type {
            0 => PayloadType {
                id: 0,
                name: "PCMU".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            1 => PayloadType {
                id: 1,
                name: "reserved (previously FS-1016 CELP)".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            2 => PayloadType {
                id: 2,
                name: "reserved (previously FS-1016 CELP)".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            3 => PayloadType {
                id: 3,
                name: "GSM".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            4 => PayloadType {
                id: 4,
                name: "G723".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            5 => PayloadType {
                id: 5,
                name: "DVI4".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            6 => PayloadType {
                id: 6,
                name: "DVI4".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            7 => PayloadType {
                id: 7,
                name: "LPC".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            8 => PayloadType {
                id: 8,
                name: "PCMA".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            9 => PayloadType {
                id: 9,
                name: "G722".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            10 => PayloadType {
                id: 10,
                name: "L16".to_string(),
                clock_rate_in_hz: Some(44100),
                media_type: Audio,
            },
            11 => PayloadType {
                id: 11,
                name: "L16".to_string(),
                clock_rate_in_hz: Some(44100),
                media_type: Audio,
            },
            12 => PayloadType {
                id: 12,
                name: "QCELP".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            13 => PayloadType {
                id: 13,
                name: "CN".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            14 => PayloadType {
                id: 14,
                name: "MPA".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: Audio,
            },
            15 => PayloadType {
                id: 15,
                name: "G728".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            16 => PayloadType {
                id: 16,
                name: "DVI4".to_string(),
                clock_rate_in_hz: Some(11025),
                media_type: Audio,
            },
            17 => PayloadType {
                id: 17,
                name: "DVI4".to_string(),
                clock_rate_in_hz: Some(22050),
                media_type: Audio,
            },
            18 => PayloadType {
                id: 18,
                name: "G729".to_string(),
                clock_rate_in_hz: Some(8000),
                media_type: Audio,
            },
            19 => PayloadType {
                id: 19,
                name: "reserved (previously CN)".to_string(),
                clock_rate_in_hz: None,
                media_type: Audio,
            },
            25 => PayloadType {
                id: 25,
                name: "CELLB".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: Video,
            },
            26 => PayloadType {
                id: 26,
                name: "JPEG".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: Video,
            },
            28 => PayloadType {
                id: 28,
                name: "nv".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: Video,
            },
            31 => PayloadType {
                id: 31,
                name: "H261".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: Video,
            },
            32 => PayloadType {
                id: 32,
                name: "MPV".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: Video,
            },
            33 => PayloadType {
                id: 33,
                name: "MP2T".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: AudioOrVideo,
            },
            34 => PayloadType {
                id: 34,
                name: "H263".to_string(),
                clock_rate_in_hz: Some(90000),
                media_type: Video,
            },
            72..=76 => PayloadType {
                id: 72,
                name: "reserved".to_string(),
                clock_rate_in_hz: None,
                media_type: AudioOrVideo,
            },
            77..=95 => PayloadType {
                id: 77,
                name: "unassigned".to_string(),
                clock_rate_in_hz: None,
                media_type: AudioOrVideo,
            },
            _ => PayloadType {
                id: payload_type,
                name: "unassigned".to_string(),
                clock_rate_in_hz: None,
                media_type: AudioOrVideo,
            },
        }
    }
}

impl std::fmt::Display for PayloadType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some(clock_rate_in_hz) = self.clock_rate_in_hz {
            write!(
                fmt,
                "Payload type: {}\n\
                Name: {}\n\
                Type: {}\n\
                Clock rate: {} Hz\n",
                self.id, self.name, self.media_type, clock_rate_in_hz
            )
        } else {
            write!(
                fmt,
                "Payload type: {}\n\
                Name: {}\n\
                Type: {}\n\
                Clock rate: undefined\n",
                self.id, self.name, self.media_type
            )
        }
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
