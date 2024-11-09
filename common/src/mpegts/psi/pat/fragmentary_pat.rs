use crate::mpegts::psi::pat::{HEADER_AFTER_SECTION_LENGTH_SIZE, HEADER_SIZE};
use crate::mpegts::psi::psi_buffer::FragmentaryPsi;
use crate::mpegts::psi::{
    ProgramSpecificInformation, ProgramSpecificInformationHeader, TableId,
    CURRENT_NEXT_INDICATOR_MASK, MAX_SECTION_LENGTH, SECTION_LENGTH_UPPER_MASK,
    SECTION_SYNTAX_INDICATOR_MASK, VERSION_NUMBER_MASK,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct FragmentaryProgramAssociationTable {
    pub header: ProgramSpecificInformationHeader,
    pub transport_stream_id: u16,
    pub payload: Vec<u8>,
    pub is_stuffed: bool,
}

impl ProgramSpecificInformation for FragmentaryProgramAssociationTable {
    fn get_header(&self) -> &ProgramSpecificInformationHeader {
        &self.header
    }

    fn get_table_id(&self) -> TableId {
        TableId::ProgramAssociationSection
    }
}

impl PartialEq for FragmentaryProgramAssociationTable {
    fn eq(&self, other: &Self) -> bool {
        let header = self.header == other.header;
        let transport_stream_id = self.transport_stream_id == other.transport_stream_id;
        let payload = self.payload == other.payload;
        let is_stuffed = self.is_stuffed == other.is_stuffed;

        header && transport_stream_id && payload && is_stuffed
    }
}

impl FragmentaryPsi for FragmentaryProgramAssociationTable {
    fn unmarshall(data: &[u8], is_pointer_field: bool) -> Option<Self> {
        if data.len() < HEADER_SIZE {
            return None;
        }

        let data = if is_pointer_field {
            let pointer_field = data[0] as usize;
            &data[pointer_field + 1..]
        } else {
            data
        };

        let header = if let Some(header) = Self::unmarshall_header(data) {
            header
        } else {
            return None;
        };

        let transport_stream_id = ((data[3] as u16) << 8) | data[4] as u16;

        let last_byte = Self::determine_last_byte(data);

        let payload = data[HEADER_SIZE + HEADER_AFTER_SECTION_LENGTH_SIZE..last_byte].to_vec();

        Some(FragmentaryProgramAssociationTable {
            header,
            transport_stream_id,
            payload,
            is_stuffed: last_byte < data.len(),
        })
    }

    fn unmarshall_header(data: &[u8]) -> Option<ProgramSpecificInformationHeader> {
        let table_id = data[0];
        let section_syntax_indicator = (data[1] & SECTION_SYNTAX_INDICATOR_MASK) != 0;
        let section_length = ((data[1] & SECTION_LENGTH_UPPER_MASK) as u16) << 8 | data[2] as u16;

        if section_length < HEADER_AFTER_SECTION_LENGTH_SIZE as u16 {
            return None;
        }

        if section_length > MAX_SECTION_LENGTH {
            return None;
        }

        let version_number = (data[5] & VERSION_NUMBER_MASK) >> 1;
        let current_next_indicator = (data[5] & CURRENT_NEXT_INDICATOR_MASK) != 0;
        let section_number = data[6];
        let last_section_number = data[7];

        Some(ProgramSpecificInformationHeader {
            table_id,
            section_syntax_indicator,
            section_length,
            version_number,
            current_next_indicator,
            section_number,
            last_section_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unmarshall_with_pointer_field() {
        let data: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, 0xB0, 0x31, 0x00, 0x14, 0xD7, 0x00, 0x00, 0x00, 0x00, 0xE0,
            0x10, 0x00, 0x01, 0xE0, 0x24, 0x00, 0x02, 0xE0, 0x25, 0x00, 0x03, 0xE0, 0x30, 0x00,
            0x04, 0xE0, 0x31, 0x00, 0x1A, 0xE0, 0x67, 0x00, 0x1C, 0xE0, 0x6F, 0x43, 0x9D, 0xE3,
            0xF1, 0x43, 0xA3, 0xE3, 0xF7, 0x43, 0xAC, 0xE4, 0x00, 0xC3, 0x69, 0xA6, 0xD8, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        // Vector to collect the payloads from each fragment
        let unmarshalled = FragmentaryProgramAssociationTable::unmarshall(&data, true).unwrap();
        assert_eq!(unmarshalled.header.table_id, 0);
        assert_eq!(unmarshalled.header.section_syntax_indicator, true);
        assert_eq!(unmarshalled.header.section_length, 49);
        assert_eq!(unmarshalled.header.current_next_indicator, true);
        assert_eq!(unmarshalled.transport_stream_id, 20);
        assert_eq!(unmarshalled.is_stuffed, true);
        assert_eq!(unmarshalled.payload.len(), 44);
    }

    #[test]
    fn test_unmarshall_without_pointer_field() {
        let data: Vec<u8> = vec![
            0x00, 0xB0, 0x31, 0x00, 0x14, 0xD7, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x10, 0x00, 0x01,
            0xE0, 0x24, 0x00, 0x02, 0xE0, 0x25, 0x00, 0x03, 0xE0, 0x30, 0x00, 0x04, 0xE0, 0x31,
            0x00, 0x1A, 0xE0, 0x67, 0x00, 0x1C, 0xE0, 0x6F, 0x43, 0x9D, 0xE3, 0xF1, 0x43, 0xA3,
            0xE3, 0xF7, 0x43, 0xAC, 0xE4, 0x00, 0xC3, 0x69, 0xA6, 0xD8, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        // Vector to collect the payloads from each fragment
        let unmarshalled = FragmentaryProgramAssociationTable::unmarshall(&data, false).unwrap();
        assert_eq!(unmarshalled.header.table_id, 0);
        assert_eq!(unmarshalled.header.section_syntax_indicator, true);
        assert_eq!(unmarshalled.header.section_length, 49);
        assert_eq!(unmarshalled.header.current_next_indicator, true);
        assert_eq!(unmarshalled.transport_stream_id, 20);
        assert_eq!(unmarshalled.is_stuffed, true);
        assert_eq!(unmarshalled.payload.len(), 44);
    }

    #[test]
    fn test_fragmentary_pat() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0xb0, 0x0d, 0x00, 0x03, 0xdf, 0x00, 0x00, 0x00, 0x23, 0xed, 0xad, 0x5a,
            0xe9, 0x7d, 0xda, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff,
        ];

        // Vector to collect the payloads from each fragment
        let unmarshalled = FragmentaryProgramAssociationTable::unmarshall(&data, true).unwrap();
        assert_eq!(unmarshalled.header.table_id, 0);
        assert_eq!(unmarshalled.header.section_syntax_indicator, true);
        assert_eq!(unmarshalled.header.section_length, 13);
        assert_eq!(unmarshalled.header.current_next_indicator, true);
        assert_eq!(unmarshalled.header.section_number, 0);
        assert_eq!(unmarshalled.header.version_number, 0x0f);
        assert_eq!(unmarshalled.header.last_section_number, 0x0);
        assert_eq!(unmarshalled.transport_stream_id, 3);
        assert_eq!(unmarshalled.is_stuffed, true);
        assert_eq!(unmarshalled.payload.len(), 8);
    }

    #[test]
    fn should_return_none_when_data_is_empty() {
        let data: Vec<u8> = vec![];
        assert_eq!(
            FragmentaryProgramAssociationTable::unmarshall(&data, false),
            None
        );
    }

    #[test]
    fn should_return_none_when_data_is_too_short() {
        let data: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            FragmentaryProgramAssociationTable::unmarshall(&data, false),
            None
        );
    }

    #[test]
    fn should_return_none_when_section_length_is_too_small() {
        let data: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            FragmentaryProgramAssociationTable::unmarshall(&data, false),
            None
        );
    }

    #[test]
    fn should_return_none_when_section_length_is_too_large() {
        let data: Vec<u8> = vec![0x00, 0x00, 0x03, 0xFE, 0x00];
        assert_eq!(
            FragmentaryProgramAssociationTable::unmarshall(&data, false),
            None
        );
    }
}
