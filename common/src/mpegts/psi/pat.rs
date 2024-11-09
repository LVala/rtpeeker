pub mod fragmentary_pat;
pub mod pat_buffer;

use serde::{Deserialize, Serialize};

const HEADER_SIZE: usize = 3;

const HEADER_AFTER_SECTION_LENGTH_SIZE: usize = 5;

const PROGRAM_SECTION_SIZE: usize = 4;

const PROGRAM_PID_UPPER_MASK: u8 = 0x1F;

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct ProgramAssociationTable {
    pub transport_stream_id: u16,
    pub programs: Vec<ProgramAssociationItem>,
    pub crc_32: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq)]
pub struct ProgramAssociationItem {
    pub program_number: u16,
    pub network_pid: Option<u16>,
    pub program_map_pid: Option<u16>,
}

impl PartialEq for ProgramAssociationTable {
    fn eq(&self, other: &Self) -> bool {
        let transport_stream_id = self.transport_stream_id == other.transport_stream_id;
        let programs = self.programs == other.programs;
        let crc_32 = self.crc_32 == other.crc_32;

        transport_stream_id && programs && crc_32
    }
}

impl PartialEq for ProgramAssociationItem {
    fn eq(&self, other: &Self) -> bool {
        let program_numbers = self.program_number == other.program_number;
        let network_pids = match (self.network_pid, other.network_pid) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        };
        let program_map_pids = match (self.program_map_pid, other.program_map_pid) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        };

        program_numbers && network_pids && program_map_pids
    }
}

impl ProgramAssociationTable {
    pub fn build(transport_stream_id: u16, data: &[u8]) -> Option<Self> {
        Some(ProgramAssociationTable {
            transport_stream_id,
            programs: ProgramAssociationTable::unmarshal_programs(data),
            crc_32: ProgramAssociationTable::unmarshal_crc_32(data),
        })
    }

    fn unmarshal_programs(data: &[u8]) -> Vec<ProgramAssociationItem> {
        let mut programs = Vec::new();
        let mut index = 0;
        while index < data.len() - 4 {
            // Skip CRC-32
            let program_number = ((data[index] as u16) << 8) | data[index + 1] as u16;
            if program_number == 0 {
                // 0xrrrnnnnn nnnnnnnn; r = reserved, n = network_pid
                let network_pid = ((data[index + 2] & PROGRAM_PID_UPPER_MASK) as u16) << 8
                    | data[index + 3] as u16;
                programs.push(ProgramAssociationItem {
                    program_number,
                    network_pid: Some(network_pid),
                    program_map_pid: None,
                });
                index += PROGRAM_SECTION_SIZE;
                continue;
            }
            // 0xrrrppppp pppppppp; r - reserved, p = program_map_pid

            let program_map_pid =
                (((data[index + 2] & PROGRAM_PID_UPPER_MASK) as u16) << 8) | data[index + 3] as u16;

            programs.push(ProgramAssociationItem {
                program_number,
                network_pid: None,
                program_map_pid: Some(program_map_pid),
            });
            index += PROGRAM_SECTION_SIZE;
        }
        programs
    }

    fn unmarshal_crc_32(data: &[u8]) -> u32 {
        let crc_32 = ((data[data.len() - 4] as u32) << 24)
            | ((data[data.len() - 3] as u32) << 16)
            | ((data[data.len() - 2] as u32) << 8)
            | data[data.len() - 1] as u32;
        crc_32
    }
}
