use crate::rtp::payload_type::PayloadType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Sdp {
    pub payload_types: HashMap<u8, PayloadType>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Sdp {
    pub fn build(raw_sdp: String) -> Option<Self> {
        use crate::rtp::payload_type::MediaType;
        use webrtc_sdp::{
            attribute_type::SdpAttribute, media_type::SdpMediaValue, parse_sdp_line, SdpLine,
            SdpType,
        };

        let mut lines = raw_sdp.lines();

        // first line should be media
        let Some(first_line) = lines.next() else {
            return None;
        };
        let Ok(SdpLine {
            sdp_type: SdpType::Media(media),
            ..
        }) = parse_sdp_line(first_line, 0)
        else {
            return None;
        };

        let media_type = match media.media {
            SdpMediaValue::Audio => MediaType::Audio,
            SdpMediaValue::Video => MediaType::Video,
            _ => {
                return None;
            }
        };

        let payload_types = lines
            .filter_map(|line| {
                let Ok(SdpLine {
                    sdp_type: SdpType::Attribute(SdpAttribute::Rtpmap(rtpmap)),
                    ..
                }) = parse_sdp_line(line, 1)
                else {
                    return None;
                };

                let pt = PayloadType {
                    id: rtpmap.payload_type,
                    name: rtpmap.codec_name,
                    clock_rate: Some(rtpmap.frequency),
                    media_type,
                };

                Some((pt.id, pt))
            })
            .collect();

        Some(Self { payload_types })
    }
}
