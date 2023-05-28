use crate::sniffer::rtp::PayloadType;

pub fn from(payload_type: u8) -> PayloadType {
    let payload = match payload_type {
        0 => PayloadType {
            id: 0,
            name: "PCMU".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        1 => PayloadType {
            id: 1,
            name: "reserved (previously FS-1016 CELP)".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        2 => PayloadType {
            id: 2,
            name: "reserved (previously FS-1016 CELP)".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        3 => PayloadType {
            id: 3,
            name: "GSM".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        4 => PayloadType {
            id: 4,
            name: "G723".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        5 => PayloadType {
            id: 5,
            name: "DVI4".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        6 => PayloadType {
            id: 6,
            name: "DVI4".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        7 => PayloadType {
            id: 7,
            name: "LPC".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        8 => PayloadType {
            id: 8,
            name: "PCMA".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        9 => PayloadType {
            id: 9,
            name: "G722".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        10 => PayloadType {
            id: 10,
            name: "L16".to_string(),
            clock_rate_in_hz: Some(44100),
        },
        11 => PayloadType {
            id: 11,
            name: "L16".to_string(),
            clock_rate_in_hz: Some(44100),
        },
        12 => PayloadType {
            id: 12,
            name: "QCELP".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        13 => PayloadType {
            id: 13,
            name: "CN".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        14 => PayloadType {
            id: 14,
            name: "MPA".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        15 => PayloadType {
            id: 15,
            name: "G728".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        16 => PayloadType {
            id: 16,
            name: "DVI4".to_string(),
            clock_rate_in_hz: Some(11025),
        },
        17 => PayloadType {
            id: 17,
            name: "DVI4".to_string(),
            clock_rate_in_hz: Some(22050),
        },
        18 => PayloadType {
            id: 18,
            name: "G729".to_string(),
            clock_rate_in_hz: Some(8000),
        },
        19 => PayloadType {
            id: 19,
            name: "reserved (previously CN)".to_string(),
            clock_rate_in_hz: None,
        },
        25 => PayloadType {
            id: 25,
            name: "CELLB".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        26 => PayloadType {
            id: 26,
            name: "JPEG".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        28 => PayloadType {
            id: 28,
            name: "nv".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        31 => PayloadType {
            id: 31,
            name: "H261".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        32 => PayloadType {
            id: 32,
            name: "MPV".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        33 => PayloadType {
            id: 33,
            name: "MP2T".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        34 => PayloadType {
            id: 34,
            name: "H263".to_string(),
            clock_rate_in_hz: Some(90000),
        },
        72..=76 => PayloadType {
            id: 72,
            name: "reserved".to_string(),
            clock_rate_in_hz: None,
        },
        77..=95 => PayloadType {
            id: 77,
            name: "unassigned".to_string(),
            clock_rate_in_hz: None,
        },
        _ => PayloadType {
            id: payload_type,
            name: payload_type.to_string(),
            clock_rate_in_hz: None,
        },
    };
    payload
}
