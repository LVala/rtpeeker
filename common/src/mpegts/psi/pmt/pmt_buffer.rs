use serde::{Deserialize, Serialize};

use crate::mpegts::psi::pmt::fragmentary_pmt::FragmentaryProgramMapTable;
use crate::mpegts::psi::pmt::{PmtFields, ProgramMapTable};
use crate::mpegts::psi::psi_buffer::PsiBuffer;
#[derive(Serialize, Deserialize, Debug)]
pub struct PmtBuffer {
    last_section_number: u8,
    pmt_fragments: Vec<FragmentaryProgramMapTable>,
}

impl PsiBuffer<ProgramMapTable, FragmentaryProgramMapTable> for PmtBuffer {
    fn new(last_section_number: u8) -> Self {
        PmtBuffer {
            last_section_number,
            pmt_fragments: Vec::new(),
        }
    }

    fn is_complete(&self) -> bool {
        self.pmt_fragments.len() as u8 == self.last_section_number + 1
    }

    fn last_section_number(&self) -> u8 {
        self.last_section_number
    }

    fn set_last_section_number(&mut self, last_section_number: u8) {
        if self.last_section_number == 0 {
            self.last_section_number = last_section_number;
        }
    }

    fn add_fragment(&mut self, fragment: FragmentaryProgramMapTable) {
        self.pmt_fragments.push(fragment);
    }

    fn get_fragments(&self) -> &Vec<FragmentaryProgramMapTable> {
        &self.pmt_fragments
    }

    fn build(&self) -> Option<ProgramMapTable> {
        if !self.is_complete() {
            return None;
        }

        let (cumulated_payload, cumulated_descriptors_payload) = self.accumulator();
        let fields = PmtFields {
            program_number: self.pmt_fragments[0].fields.program_number,
            pcr_pid: self.pmt_fragments[0].fields.pcr_pid,
            program_info_length: self.pmt_fragments[0].fields.program_info_length,
        };

        ProgramMapTable::build(fields, &cumulated_descriptors_payload, &cumulated_payload)
    }
}

impl PmtBuffer {
    pub fn get_program_number(&self) -> u16 {
        self.pmt_fragments[0].fields.program_number
    }

    pub fn is_fragment_inside(&self, fragment: &FragmentaryProgramMapTable) -> bool {
        (self.pmt_fragments.len() as u8) >= fragment.header.section_number
            && self.pmt_fragments[0].fields.program_number == fragment.fields.program_number
    }

    fn accumulator(&self) -> (Vec<u8>, Vec<u8>) {
        let cumulated_payload = self
            .pmt_fragments
            .iter()
            .fold(Vec::new(), |mut acc, fragment| {
                acc.extend_from_slice(&fragment.payload);
                acc
            });

        let cumulated_descriptors_payload =
            self.pmt_fragments
                .iter()
                .fold(Vec::new(), |mut acc, fragment| {
                    acc.extend_from_slice(&fragment.descriptors_payload);
                    acc
                });

        (cumulated_payload, cumulated_descriptors_payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpegts::descriptors::audio_stream::AudioStreamDescriptor;
    use crate::mpegts::descriptors::data_stream_alignment_descriptor::AlignmentType::SliceOrVideoAccessUnit;
    use crate::mpegts::descriptors::data_stream_alignment_descriptor::DataStreamAlignmentDescriptor;
    use crate::mpegts::descriptors::iso_639_language_descriptor::AudioType::{
        Undefined, VisualImpairedCommentary,
    };
    use crate::mpegts::descriptors::iso_639_language_descriptor::{
        Iso639LanguageDescriptor, Section,
    };
    use crate::mpegts::descriptors::maximum_bitrate_descriptor::MaximumBitrateDescriptor;
    use crate::mpegts::descriptors::multiplex_buffer_utilization_descriptor::MultiplexBufferUtilizationDescriptor;
    use crate::mpegts::descriptors::registration_descriptor::RegistrationDescriptor;
    use crate::mpegts::descriptors::tags::DescriptorTag::{
        AudioStreamDescriptorTag, AvcVideoDescriptorTag, DataStreamAlignmentDescriptorTag,
        Iso639LanguageDescriptorTag, MaximumBitrateDescriptorTag,
        MultiplexBufferUtilizationDescriptorTag, RegistrationDescriptorTag,
        VideoStreamDescriptorTag,
    };
    use crate::mpegts::descriptors::video_stream::VideoStreamDescriptor;
    use crate::mpegts::descriptors::{avc_video_descriptor, DescriptorHeader, Descriptors};
    use crate::mpegts::psi::pmt::stream_types::StreamTypes::{
        AVCVideoStreamAsDefinedInItuTH264OrIsoIec1449610Video, IsoIec111723Audio,
        RecItuTH2220OrIsoIec138181PESPackets, RecItuTH2220OrIsoIec138181PrivateSections,
    };
    use crate::mpegts::psi::pmt::{ElementaryStreamInfo, PmtFields};
    use crate::mpegts::psi::psi_buffer::FragmentaryPsi;

    #[test]
    fn test_pmt_buffer_with_one_fragment() {
        let data = vec![
            0x00, 0x02, 0xb0, 0x90, 0x00, 0x21, 0xd5, 0x00, 0x00, 0xe2, 0x5a, 0xf0, 0x0b, 0x0e,
            0x03, 0xc0, 0x00, 0x00, 0x0c, 0x04, 0x80, 0xb4, 0x81, 0x68, 0x1b, 0xe2, 0x5a, 0xf0,
            0x16, 0x52, 0x01, 0x02, 0x0e, 0x03, 0xc0, 0x00, 0x00, 0x02, 0x03, 0x1a, 0x44, 0x5f,
            0x06, 0x01, 0x02, 0x28, 0x04, 0x4d, 0x40, 0x28, 0x3f, 0x03, 0xe2, 0x5b, 0xf0, 0x11,
            0x52, 0x01, 0x03, 0x0e, 0x03, 0xc0, 0x00, 0x00, 0x0a, 0x04, 0x70, 0x6f, 0x6c, 0x00,
            0x03, 0x01, 0x67, 0x05, 0xe2, 0x5f, 0xf0, 0x0d, 0x52, 0x01, 0x07, 0x0e, 0x03, 0xc0,
            0x00, 0x00, 0x6f, 0x03, 0x00, 0x10, 0xe0, 0x06, 0xe2, 0x5e, 0xf0, 0x12, 0x52, 0x01,
            0x06, 0x0e, 0x03, 0xc0, 0x00, 0x00, 0x59, 0x08, 0x70, 0x6f, 0x6c, 0x10, 0x00, 0x02,
            0x00, 0x02, 0x06, 0xe2, 0x60, 0xf0, 0x19, 0x52, 0x01, 0x08, 0x0e, 0x03, 0xc0, 0x00,
            0x00, 0x0a, 0x04, 0x61, 0x75, 0x78, 0x03, 0x05, 0x04, 0x45, 0x41, 0x43, 0x33, 0x7a,
            0x03, 0xc0, 0x92, 0x10, 0x33, 0x59, 0xb6, 0x88, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ];

        let mut buffer = PmtBuffer::new(0);
        let fragment = FragmentaryProgramMapTable::unmarshall(&data, true).unwrap();
        buffer.add_fragment(fragment);
        let fields = PmtFields {
            program_number: 33,
            pcr_pid: 0x025a,
            program_info_length: 11,
        };

        let right = Some(ProgramMapTable {
            fields,
            descriptors: vec![
                Descriptors::MaximumBitrateDescriptor(MaximumBitrateDescriptor {
                    header: DescriptorHeader {
                        descriptor_tag: MaximumBitrateDescriptorTag,
                        descriptor_length: 3,
                    },
                    maximum_bitrate: 0,
                }),
                Descriptors::MultiplexBufferUtilizationDescriptor(
                    MultiplexBufferUtilizationDescriptor {
                        header: DescriptorHeader {
                            descriptor_tag: MultiplexBufferUtilizationDescriptorTag,
                            descriptor_length: 4,
                        },
                        bound_valid_flag: true,
                        ltw_offset_lower_bound: Some(180),
                        ltw_offset_upper_bound: Some(360),
                    },
                ),
            ],
            elementary_streams_info: vec![
                ElementaryStreamInfo {
                    stream_type: AVCVideoStreamAsDefinedInItuTH264OrIsoIec1449610Video,
                    elementary_pid: 602,
                    es_info_length: 22,
                    descriptors: vec![
                        Descriptors::UserPrivate(82),
                        Descriptors::MaximumBitrateDescriptor(MaximumBitrateDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: MaximumBitrateDescriptorTag,
                                descriptor_length: 3,
                            },
                            maximum_bitrate: 0,
                        }),
                        Descriptors::VideoStreamDescriptor(VideoStreamDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: VideoStreamDescriptorTag,
                                descriptor_length: 3,
                            },
                            multiple_frame_rate_flag: false,
                            frame_rate_code: 3,
                            mpeg_1_only_flag: true,
                            constrained_parameter_flag: true,
                            still_picture_flag: false,
                            profile_and_level_indication: None,
                            chroma_format: None,
                            frame_rate_extension_flag: None,
                        }),
                        Descriptors::DataStreamAlignmentDescriptor(DataStreamAlignmentDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: DataStreamAlignmentDescriptorTag,
                                descriptor_length: 1,
                            },
                            alignment_type: SliceOrVideoAccessUnit,
                        }),
                        Descriptors::AvcVideoDescriptor(avc_video_descriptor::AvcVideoDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: AvcVideoDescriptorTag,
                                descriptor_length: 4,
                            },
                            profile_idc: 77,
                            constraint_set0_flag: false,
                            constraint_set1_flag: true,
                            constraint_set2_flag: false,
                            constraint_set3_flag: false,
                            constraint_set4_flag: false,
                            constraint_set5_flag: false,
                            avc_compatible_flags: 0,
                            level_idc: 40,
                            avc_still_present: false,
                            avc_24_hour_picture_flag: true,
                            frame_packing_sei_flag: false,
                        }),
                    ],
                },
                ElementaryStreamInfo {
                    stream_type: IsoIec111723Audio,
                    elementary_pid: 603,
                    es_info_length: 17,
                    descriptors: vec![
                        Descriptors::UserPrivate(82),
                        Descriptors::MaximumBitrateDescriptor(MaximumBitrateDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: MaximumBitrateDescriptorTag,
                                descriptor_length: 3,
                            },
                            maximum_bitrate: 0,
                        }),
                        Descriptors::Iso639LanguageDescriptor(Iso639LanguageDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: Iso639LanguageDescriptorTag,
                                descriptor_length: 4,
                            },
                            section: vec![Section {
                                language_code: "pol".to_string(),
                                audio_type: Undefined,
                            }],
                        }),
                        Descriptors::AudioStreamDescriptor(AudioStreamDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: AudioStreamDescriptorTag,
                                descriptor_length: 1,
                            },
                            free_format_flag: false,
                            id: 1,
                            layer: 2,
                            variable_rate_audio_indicator: false,
                        }),
                    ],
                },
                ElementaryStreamInfo {
                    stream_type: RecItuTH2220OrIsoIec138181PrivateSections,
                    elementary_pid: 607,
                    es_info_length: 13,
                    descriptors: vec![
                        Descriptors::UserPrivate(82),
                        Descriptors::MaximumBitrateDescriptor(MaximumBitrateDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: MaximumBitrateDescriptorTag,
                                descriptor_length: 3,
                            },
                            maximum_bitrate: 0,
                        }),
                        Descriptors::UserPrivate(111),
                    ],
                },
                ElementaryStreamInfo {
                    stream_type: RecItuTH2220OrIsoIec138181PESPackets,
                    elementary_pid: 606,
                    es_info_length: 18,
                    descriptors: vec![
                        Descriptors::UserPrivate(82),
                        Descriptors::MaximumBitrateDescriptor(MaximumBitrateDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: MaximumBitrateDescriptorTag,
                                descriptor_length: 3,
                            },
                            maximum_bitrate: 0,
                        }),
                        Descriptors::UserPrivate(89),
                    ],
                },
                ElementaryStreamInfo {
                    stream_type: RecItuTH2220OrIsoIec138181PESPackets,
                    elementary_pid: 608,
                    es_info_length: 25,
                    descriptors: vec![
                        Descriptors::UserPrivate(82),
                        Descriptors::MaximumBitrateDescriptor(MaximumBitrateDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: MaximumBitrateDescriptorTag,
                                descriptor_length: 3,
                            },
                            maximum_bitrate: 0,
                        }),
                        Descriptors::Iso639LanguageDescriptor(Iso639LanguageDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: Iso639LanguageDescriptorTag,
                                descriptor_length: 4,
                            },
                            section: vec![Section {
                                language_code: "aux".to_string(),
                                audio_type: VisualImpairedCommentary,
                            }],
                        }),
                        Descriptors::RegistrationDescriptor(RegistrationDescriptor {
                            header: DescriptorHeader {
                                descriptor_tag: RegistrationDescriptorTag,
                                descriptor_length: 4,
                            },
                            format_identifier: 1161904947,
                            additional_identification_info: vec![],
                        }),
                        Descriptors::UserPrivate(122),
                    ],
                },
            ],
            crc_32: 0x3359b688,
        });

        assert_eq!(buffer.is_complete(), true);
        assert_eq!(buffer.build(), right);
    }
}
