use crate::mpegts::psi::pmt::{
    PmtFields, HEADER_AFTER_SECTION_LENGTH_SIZE, HEADER_SIZE, PCR_PID_LOWER_MASK,
    PCR_PID_UPPER_MASK, PROGRAM_INFO_LENGTH_LOWER_MASK, PROGRAM_INFO_LENGTH_UPPER_MASK,
};
use crate::mpegts::psi::psi_buffer::FragmentaryPsi;
use crate::mpegts::psi::{
    ProgramSpecificInformationHeader, CURRENT_NEXT_INDICATOR_MASK, MAX_SECTION_LENGTH,
    SECTION_LENGTH_UPPER_MASK, SECTION_SYNTAX_INDICATOR_MASK, VERSION_NUMBER_MASK,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct FragmentaryProgramMapTable {
    pub header: ProgramSpecificInformationHeader,
    pub fields: PmtFields,
    pub descriptors_payload: Vec<u8>,
    pub payload: Vec<u8>, //rest of the payload
    pub is_stuffed: bool,
}

impl PartialEq for FragmentaryProgramMapTable {
    fn eq(&self, other: &Self) -> bool {
        let header = self.header == other.header;
        let payload = self.payload == other.payload;
        let is_stuffed = self.is_stuffed == other.is_stuffed;
        let fields = self.fields == other.fields;
        let descriptors_payload = self.descriptors_payload == other.descriptors_payload;

        header && payload && is_stuffed && fields && descriptors_payload
    }
}

impl FragmentaryPsi for FragmentaryProgramMapTable {
    fn unmarshall(data: &[u8], is_pointer_field: bool) -> Option<Self>
    where
        Self: Sized,
    {
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
        let full_header_size = HEADER_SIZE + HEADER_AFTER_SECTION_LENGTH_SIZE;

        let program_number = ((data[3] as u16) << 8) | data[4] as u16;
        let pcr_pid = (((data[8] & PCR_PID_UPPER_MASK as u8) as u16) << 8)
            | (data[9] & PCR_PID_LOWER_MASK as u8) as u16;
        let program_info_length = (((data[10] & PROGRAM_INFO_LENGTH_UPPER_MASK as u8) as u16) << 8)
            | (data[11] & PROGRAM_INFO_LENGTH_LOWER_MASK as u8) as u16;
        let descriptors_payload =
            data[full_header_size..full_header_size + program_info_length as usize].to_vec();
        let last_byte = Self::determine_last_byte(data);
        let payload = data[full_header_size + program_info_length as usize..last_byte].to_vec();
        let fields = PmtFields {
            program_number,
            pcr_pid,
            program_info_length,
        };

        Some(FragmentaryProgramMapTable {
            header,
            fields,
            descriptors_payload,
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
    fn test_unmarshall() {
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
        let fragmentary_pmt = FragmentaryProgramMapTable::unmarshall(&data, true).unwrap();
        assert_eq!(fragmentary_pmt.header.table_id, 0x02);
        assert_eq!(fragmentary_pmt.header.section_syntax_indicator, true);
        assert_eq!(fragmentary_pmt.header.section_length, 144);
        assert_eq!(fragmentary_pmt.header.version_number, 0x0a);
        assert_eq!(fragmentary_pmt.header.current_next_indicator, true);
        assert_eq!(fragmentary_pmt.header.section_number, 0);
        assert_eq!(fragmentary_pmt.header.last_section_number, 0);
        assert_eq!(fragmentary_pmt.fields.program_number, 0x0021);
        assert_eq!(fragmentary_pmt.fields.pcr_pid, 0x025a);
        assert_eq!(fragmentary_pmt.fields.program_info_length, 11);
        assert_eq!(
            fragmentary_pmt.descriptors_payload,
            vec![0x0e, 0x03, 0xc0, 0x00, 0x00, 0x0c, 0x04, 0x80, 0xb4, 0x81, 0x68,]
        );
        assert_eq!(
            fragmentary_pmt.payload,
            vec![
                0x1b, 0xe2, 0x5a, 0xf0, 0x16, 0x52, 0x01, 0x02, 0x0e, 0x03, 0xc0, 0x00, 0x00, 0x02,
                0x03, 0x1a, 0x44, 0x5f, 0x06, 0x01, 0x02, 0x28, 0x04, 0x4d, 0x40, 0x28, 0x3f, 0x03,
                0xe2, 0x5b, 0xf0, 0x11, 0x52, 0x01, 0x03, 0x0e, 0x03, 0xc0, 0x00, 0x00, 0x0a, 0x04,
                0x70, 0x6f, 0x6c, 0x00, 0x03, 0x01, 0x67, 0x05, 0xe2, 0x5f, 0xf0, 0x0d, 0x52, 0x01,
                0x07, 0x0e, 0x03, 0xc0, 0x00, 0x00, 0x6f, 0x03, 0x00, 0x10, 0xe0, 0x06, 0xe2, 0x5e,
                0xf0, 0x12, 0x52, 0x01, 0x06, 0x0e, 0x03, 0xc0, 0x00, 0x00, 0x59, 0x08, 0x70, 0x6f,
                0x6c, 0x10, 0x00, 0x02, 0x00, 0x02, 0x06, 0xe2, 0x60, 0xf0, 0x19, 0x52, 0x01, 0x08,
                0x0e, 0x03, 0xc0, 0x00, 0x00, 0x0a, 0x04, 0x61, 0x75, 0x78, 0x03, 0x05, 0x04, 0x45,
                0x41, 0x43, 0x33, 0x7a, 0x03, 0xc0, 0x92, 0x10, 0x33, 0x59, 0xb6, 0x88,
            ]
        );
    }
}
