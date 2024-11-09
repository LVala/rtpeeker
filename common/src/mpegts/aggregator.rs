use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::psi::pat::fragmentary_pat::FragmentaryProgramAssociationTable;
use super::psi::pmt::fragmentary_pmt::FragmentaryProgramMapTable;
use super::psi::psi_buffer::PsiBuffer;
use super::psi::{pat::pat_buffer::PatBuffer, pmt::pmt_buffer::PmtBuffer};

#[derive(Serialize, Deserialize, Debug)]
pub struct MpegtsAggregator {
    pub pat_buffer: PatBuffer,
    pub pmt_buffers: HashMap<u16, PmtBuffer>,
}

impl MpegtsAggregator {
    pub fn new() -> Self {
        MpegtsAggregator {
            pat_buffer: PatBuffer::new(0),
            pmt_buffers: HashMap::new(),
        }
    }

    pub fn add_pat(&mut self, fragment: FragmentaryProgramAssociationTable) {
        self.pat_buffer.add_fragment(fragment);
    }

    pub fn add_pmt(&mut self, pmt_pid: u16, fragment: FragmentaryProgramMapTable) {
        if let Some(pmt_buffer) = self.pmt_buffers.get_mut(&pmt_pid) {
            if pmt_buffer.is_complete() {
                return;
            }

            if !pmt_buffer.is_fragment_inside(&fragment) {
                pmt_buffer.add_fragment(fragment);
            }
            return;
        }

        let mut pmt_buffer = PmtBuffer::new(fragment.header.last_section_number);
        pmt_buffer.add_fragment(fragment);
        self.pmt_buffers.insert(pmt_pid, pmt_buffer);
    }

    pub fn get_pat(&self) -> Option<super::psi::pat::ProgramAssociationTable> {
        self.pat_buffer.build()
    }

    pub fn get_pmt(&self, pid: u16) -> Option<super::psi::pmt::ProgramMapTable> {
        if let Some(pmt_buffer) = self.pmt_buffers.get(&pid) {
            pmt_buffer.build()
        } else {
            None
        }
    }

    pub fn is_pat_complete(&self) -> bool {
        self.pat_buffer.is_complete()
    }

    pub fn is_pmt_complete(&self, pid: u16) -> bool {
        if let Some(pmt_buffer) = self.pmt_buffers.get(&pid) {
            pmt_buffer.is_complete()
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.pat_buffer.clear();
        self.pmt_buffers.clear();
    }
}
