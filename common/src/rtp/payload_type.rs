use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MediaType {
    Audio,
    Video,
    AudioOrVideo,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Self::Audio => "audio",
            Self::Video => "video",
            Self::AudioOrVideo => "audio/video",
        };

        write!(f, "{}", res)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PayloadType {
    pub id: u8,
    pub name: String,
    pub clock_rate: Option<u32>,
    pub media_type: MediaType,
}

impl PayloadType {
    pub fn new(id: u8) -> Self {
        let (name, media_type, freq) = id_to_info(id);
        let clock_rate = if freq != 0 { Some(freq) } else { None };

        Self {
            id,
            name: name.to_string(),
            media_type,
            clock_rate,
        }
    }
}

impl fmt::Display for PayloadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clock_rate = if let Some(freq) = self.clock_rate {
            format!("{} Hz", freq)
        } else {
            String::new()
        };
        let payload_type = format!("Payload type: {} {} {} {}", self.id, self.name, self.media_type, clock_rate);

        write!(
            f,
            "{}",
            payload_type
        )
    }
}

fn id_to_info(id: u8) -> (&'static str, MediaType, u32) {
    use MediaType::*;

    match id {
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
    }
}
