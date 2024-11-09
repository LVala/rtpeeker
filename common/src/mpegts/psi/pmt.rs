pub mod fragmentary_pmt;
pub mod pmt_buffer;
mod stream_types;

use crate::mpegts::descriptors::Descriptors;
use crate::mpegts::psi::pmt::stream_types::StreamTypes;
use serde::{Deserialize, Serialize};

pub const HEADER_AFTER_SECTION_LENGTH_SIZE: usize = 9;

pub const HEADER_SIZE: usize = 3;

pub const PCR_PID_UPPER_MASK: usize = 0x1F;

pub const PCR_PID_LOWER_MASK: usize = 0xFF;

pub const PROGRAM_INFO_LENGTH_UPPER_MASK: usize = 0x0F;

pub const PROGRAM_INFO_LENGTH_LOWER_MASK: usize = 0xFF;

pub const ELEMENTARY_PID_UPPER_MASK: usize = 0x1F;

pub const ELEMENTARY_PID_LOWER_MASK: usize = 0xFF;

pub const ES_INFO_LENGTH_UPPER_MASK: usize = 0x0F;
pub const ES_INFO_LENGTH_LOWER_MASK: usize = 0xFF;

pub const STREAM_LENGTH: usize = 5;

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct ProgramMapTable {
    pub fields: PmtFields,
    pub descriptors: Vec<Descriptors>,
    pub elementary_streams_info: Vec<ElementaryStreamInfo>,
    pub crc_32: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct PmtFields {
    pub program_number: u16,
    pub pcr_pid: u16,
    pub program_info_length: u16,
}

impl PartialEq for ProgramMapTable {
    fn eq(&self, other: &Self) -> bool {
        let fields = self.fields == other.fields;
        let descriptors = self.descriptors == other.descriptors;
        let elementary_streams_info = self.elementary_streams_info == other.elementary_streams_info;
        let crc_32 = self.crc_32 == other.crc_32;

        fields && descriptors && elementary_streams_info && crc_32
    }
}

impl PartialEq for PmtFields {
    fn eq(&self, other: &Self) -> bool {
        let program_number = self.program_number == other.program_number;
        let pcr_pid = self.pcr_pid == other.pcr_pid;
        let program_info_length = self.program_info_length == other.program_info_length;

        program_number && pcr_pid && program_info_length
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct ElementaryStreamInfo {
    pub stream_type: StreamTypes, // table is defined on page 55 of H.222.0 (03/2017)
    pub elementary_pid: u16,
    pub es_info_length: u16,
    pub descriptors: Vec<Descriptors>,
}

impl PartialEq for ElementaryStreamInfo {
    fn eq(&self, other: &Self) -> bool {
        let stream_type = self.stream_type == other.stream_type;
        let elementary_pid = self.elementary_pid == other.elementary_pid;
        let es_info_length = self.es_info_length == other.es_info_length;
        let descriptors = self.descriptors == other.descriptors;

        stream_type && elementary_pid && es_info_length && descriptors
    }
}

impl ProgramMapTable {
    fn build(
        fields: PmtFields,
        descriptors_payload: &[u8],
        payload: &[u8],
    ) -> Option<ProgramMapTable> {
        Some(ProgramMapTable {
            fields,
            descriptors: ProgramMapTable::unmarshal_descriptors(descriptors_payload),
            elementary_streams_info: ProgramMapTable::unmarshal_elementary_streams_info(payload),
            crc_32: ProgramMapTable::unmarshal_crc_32(payload),
        })
    }

    fn unmarshal_descriptors(data: &[u8]) -> Vec<Descriptors> {
        Descriptors::unmarshall_many(data)
    }

    fn unmarshal_elementary_streams_info(data: &[u8]) -> Vec<ElementaryStreamInfo> {
        let mut elementary_streams_info = Vec::new();
        let mut index = 0;

        while index + STREAM_LENGTH <= data.len() - 4 {
            // Skip CRC-32
            let stream_type = data[index];
            let elementary_pid = (((data[index + 1] & ELEMENTARY_PID_UPPER_MASK as u8) as u16)
                << 8)
                | (data[index + 2] & ELEMENTARY_PID_LOWER_MASK as u8) as u16;
            let es_info_length = (((data[index + 3] & ES_INFO_LENGTH_UPPER_MASK as u8) as u16)
                << 8)
                | (data[index + 4] & ES_INFO_LENGTH_LOWER_MASK as u8) as u16;
            elementary_streams_info.push(ElementaryStreamInfo {
                stream_type: StreamTypes::from(stream_type),
                elementary_pid,
                es_info_length,
                descriptors: Descriptors::unmarshall_many(
                    &data[index + STREAM_LENGTH..index + STREAM_LENGTH + es_info_length as usize],
                ),
            });

            index += STREAM_LENGTH + es_info_length as usize;
        }

        elementary_streams_info
    }

    fn unmarshal_crc_32(data: &[u8]) -> u32 {
        let crc_32_index = data.len() - 4;
        ((data[crc_32_index] as u32) << 24)
            | ((data[crc_32_index + 1] as u32) << 16)
            | ((data[crc_32_index + 2] as u32) << 8)
            | data[crc_32_index + 3] as u32
    }
}
