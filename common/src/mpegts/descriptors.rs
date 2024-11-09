pub mod audio_stream;
pub mod avc_video_descriptor;
pub mod ca_descriptor;
pub mod copyright_descriptor;
pub mod data_stream_alignment_descriptor;
pub mod hierarchy;
pub mod iso_639_language_descriptor;
pub mod maximum_bitrate_descriptor;
pub mod multiplex_buffer_utilization_descriptor;
pub mod private_data_indicator_descriptor;
pub mod registration_descriptor;
pub mod std_descriptor;
pub mod system_clock_descriptor;
pub mod tags;
pub mod target_background_grid_descriptor;
pub mod video_stream;
pub mod video_window_descriptor;

use crate::mpegts::descriptors::audio_stream::AudioStreamDescriptor;
use crate::mpegts::descriptors::avc_video_descriptor::AvcVideoDescriptor;
use crate::mpegts::descriptors::ca_descriptor::CaDescriptor;
use crate::mpegts::descriptors::copyright_descriptor::CopyrightDescriptor;
use crate::mpegts::descriptors::data_stream_alignment_descriptor::DataStreamAlignmentDescriptor;
use crate::mpegts::descriptors::hierarchy::HierarchyDescriptor;
use crate::mpegts::descriptors::iso_639_language_descriptor::Iso639LanguageDescriptor;
use crate::mpegts::descriptors::maximum_bitrate_descriptor::MaximumBitrateDescriptor;
use crate::mpegts::descriptors::multiplex_buffer_utilization_descriptor::MultiplexBufferUtilizationDescriptor;
use crate::mpegts::descriptors::private_data_indicator_descriptor::PrivateDataIndicatorDescriptor;
use crate::mpegts::descriptors::registration_descriptor::RegistrationDescriptor;
use crate::mpegts::descriptors::std_descriptor::StdDescriptor;
use crate::mpegts::descriptors::system_clock_descriptor::SystemClockDescriptor;
use crate::mpegts::descriptors::tags::DescriptorTag;
use crate::mpegts::descriptors::target_background_grid_descriptor::TargetBackgroundGridDescriptor;
use crate::mpegts::descriptors::video_stream::VideoStreamDescriptor;
use crate::mpegts::descriptors::video_window_descriptor::VideoWindowDescriptor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

const HEADER_SIZE: u8 = 2;

pub trait ParsableDescriptor<T>: Debug {
    fn descriptor_tag(&self) -> u8;
    fn descriptor_length(&self) -> u8;
    fn unmarshall(header: DescriptorHeader, data: &[u8]) -> Option<T>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub enum Descriptors {
    VideoStreamDescriptor(VideoStreamDescriptor),
    AudioStreamDescriptor(AudioStreamDescriptor),
    HierarchyDescriptor(HierarchyDescriptor),
    RegistrationDescriptor(RegistrationDescriptor),
    TargetBackgroundGridDescriptor(TargetBackgroundGridDescriptor),
    VideoWindowDescriptor(VideoWindowDescriptor),
    CaDescriptor(CaDescriptor),
    SystemClockDescriptor(SystemClockDescriptor),
    MaximumBitrateDescriptor(MaximumBitrateDescriptor),
    CopyrightDescriptor(CopyrightDescriptor),
    MultiplexBufferUtilizationDescriptor(MultiplexBufferUtilizationDescriptor),
    PrivateDataIndicatorDescriptor(PrivateDataIndicatorDescriptor),
    StdDescriptor(StdDescriptor),
    DataStreamAlignmentDescriptor(DataStreamAlignmentDescriptor),
    AvcVideoDescriptor(AvcVideoDescriptor),
    Iso639LanguageDescriptor(Iso639LanguageDescriptor),
    UserPrivate(u8),
    Unknown,
}

impl std::fmt::Display for Descriptors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Descriptors::VideoStreamDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::AudioStreamDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::HierarchyDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::RegistrationDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::TargetBackgroundGridDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::VideoWindowDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::CaDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::SystemClockDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::MaximumBitrateDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::CopyrightDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::MultiplexBufferUtilizationDescriptor(descriptor) => {
                write!(f, "{}", descriptor)
            }
            Descriptors::PrivateDataIndicatorDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::StdDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::DataStreamAlignmentDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::AvcVideoDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::Iso639LanguageDescriptor(descriptor) => write!(f, "{}", descriptor),
            Descriptors::UserPrivate(data) => write!(f, "User Private: {}", data),
            Descriptors::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Descriptors {
    pub fn unmarshall(data: &[u8]) -> Option<Self> {
        let header = DescriptorHeader::unmarshall(data);
        let payload = &data[2..];
        match header.descriptor_tag {
            DescriptorTag::VideoStreamDescriptorTag => {
                VideoStreamDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::VideoStreamDescriptor(descriptor))
            }
            DescriptorTag::AudioStreamDescriptorTag => {
                AudioStreamDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::AudioStreamDescriptor(descriptor))
            }
            DescriptorTag::HierarchyDescriptorTag => {
                HierarchyDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::HierarchyDescriptor(descriptor))
            }
            DescriptorTag::RegistrationDescriptorTag => {
                RegistrationDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::RegistrationDescriptor(descriptor))
            }
            DescriptorTag::TargetBackgroundGridDescriptorTag => {
                TargetBackgroundGridDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::TargetBackgroundGridDescriptor(descriptor))
            }
            DescriptorTag::VideoWindowDescriptorTag => {
                VideoWindowDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::VideoWindowDescriptor(descriptor))
            }
            DescriptorTag::CaDescriptorTag => CaDescriptor::unmarshall(header, payload)
                .map(|descriptor| Descriptors::CaDescriptor(descriptor)),
            DescriptorTag::SystemClockDescriptorTag => {
                SystemClockDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::SystemClockDescriptor(descriptor))
            }
            DescriptorTag::MaximumBitrateDescriptorTag => {
                MaximumBitrateDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::MaximumBitrateDescriptor(descriptor))
            }
            DescriptorTag::CopyrightDescriptorTag => {
                CopyrightDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::CopyrightDescriptor(descriptor))
            }
            DescriptorTag::MultiplexBufferUtilizationDescriptorTag => {
                MultiplexBufferUtilizationDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::MultiplexBufferUtilizationDescriptor(descriptor))
            }
            DescriptorTag::PrivateDataIndicatorDescriptorTag => {
                PrivateDataIndicatorDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::PrivateDataIndicatorDescriptor(descriptor))
            }
            DescriptorTag::StdDescriptorTag => StdDescriptor::unmarshall(header, payload)
                .map(|descriptor| Descriptors::StdDescriptor(descriptor)),
            DescriptorTag::DataStreamAlignmentDescriptorTag => {
                DataStreamAlignmentDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::DataStreamAlignmentDescriptor(descriptor))
            }
            DescriptorTag::AvcVideoDescriptorTag => AvcVideoDescriptor::unmarshall(header, payload)
                .map(|descriptor| Descriptors::AvcVideoDescriptor(descriptor)),
            DescriptorTag::Iso639LanguageDescriptorTag => {
                Iso639LanguageDescriptor::unmarshall(header, payload)
                    .map(|descriptor| Descriptors::Iso639LanguageDescriptor(descriptor))
            }
            DescriptorTag::UserPrivate => Some(Descriptors::UserPrivate(data[0])),
            _ => Some(Descriptors::Unknown),
        }
    }
    pub fn unmarshall_many(data: &[u8]) -> Vec<Self> {
        let mut descriptors = Vec::new();
        let mut offset = 0;
        while offset < data.len() {
            let header = DescriptorHeader::unmarshall(&data[offset..]);
            Self::unmarshall(
                &data[offset..(header.descriptor_length + HEADER_SIZE) as usize + offset],
            )
            .map(|descriptor| {
                descriptors.push(descriptor);
            });
            offset += (HEADER_SIZE + header.descriptor_length) as usize;
        }
        descriptors
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct DescriptorHeader {
    pub descriptor_tag: DescriptorTag,
    pub descriptor_length: u8,
}

impl DescriptorHeader {
    pub fn unmarshall(data: &[u8]) -> Self {
        let descriptor_tag = DescriptorTag::from(data[0]);
        let descriptor_length = data[1];

        DescriptorHeader {
            descriptor_tag,
            descriptor_length,
        }
    }
}

impl PartialEq for Descriptors {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Descriptors::VideoStreamDescriptor(a), Descriptors::VideoStreamDescriptor(b)) => {
                a == b
            }
            (Descriptors::AudioStreamDescriptor(a), Descriptors::AudioStreamDescriptor(b)) => {
                a == b
            }
            (Descriptors::HierarchyDescriptor(a), Descriptors::HierarchyDescriptor(b)) => a == b,
            (Descriptors::RegistrationDescriptor(a), Descriptors::RegistrationDescriptor(b)) => {
                a == b
            }
            (
                Descriptors::TargetBackgroundGridDescriptor(a),
                Descriptors::TargetBackgroundGridDescriptor(b),
            ) => a == b,
            (Descriptors::VideoWindowDescriptor(a), Descriptors::VideoWindowDescriptor(b)) => {
                a == b
            }
            (Descriptors::CaDescriptor(a), Descriptors::CaDescriptor(b)) => a == b,
            (Descriptors::SystemClockDescriptor(a), Descriptors::SystemClockDescriptor(b)) => {
                a == b
            }
            (
                Descriptors::MaximumBitrateDescriptor(a),
                Descriptors::MaximumBitrateDescriptor(b),
            ) => a == b,
            (Descriptors::CopyrightDescriptor(a), Descriptors::CopyrightDescriptor(b)) => a == b,
            (
                Descriptors::MultiplexBufferUtilizationDescriptor(a),
                Descriptors::MultiplexBufferUtilizationDescriptor(b),
            ) => a == b,
            (
                Descriptors::PrivateDataIndicatorDescriptor(a),
                Descriptors::PrivateDataIndicatorDescriptor(b),
            ) => a == b,
            (Descriptors::StdDescriptor(a), Descriptors::StdDescriptor(b)) => a == b,
            (
                Descriptors::DataStreamAlignmentDescriptor(a),
                Descriptors::DataStreamAlignmentDescriptor(b),
            ) => a == b,
            (Descriptors::AvcVideoDescriptor(a), Descriptors::AvcVideoDescriptor(b)) => a == b,
            (
                Descriptors::Iso639LanguageDescriptor(a),
                Descriptors::Iso639LanguageDescriptor(b),
            ) => a == b,
            (Descriptors::UserPrivate(a), Descriptors::UserPrivate(b)) => a == b,
            (Descriptors::Unknown, Descriptors::Unknown) => true,
            _ => false,
        }
    }
}

impl PartialEq for DescriptorHeader {
    fn eq(&self, other: &Self) -> bool {
        let descriptor_tag = self.descriptor_tag == other.descriptor_tag;
        let descriptor_length = self.descriptor_length == other.descriptor_length;

        descriptor_tag && descriptor_length
    }
}
