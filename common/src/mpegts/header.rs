use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub transport_error_indicator: bool,
    pub payload_unit_start_indicator: bool,
    pub transport_priority: bool,
    pub pid: PIDTable,
    pub transport_scrambling_control: TransportScramblingControl,
    pub adaptation_field_control: AdaptationFieldControl,
    pub continuity_counter: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PIDTable {
    ProgramAssociation,
    ConditionalAccess,
    TransportStreamDescription,
    IPMPControlInformation,
    AdaptiveStreamingInformation,
    PID(u16),
    NullPacket,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransportScramblingControl {
    NotScrambled,
    UserDefined(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AdaptationFieldControl {
    PayloadOnly,
    AdaptationFieldOnly,
    AdaptationFieldAndPaylod,
}

impl std::fmt::Display for PIDTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PIDTable::NullPacket => write!(f, "Null Packet"),
            PIDTable::ProgramAssociation => write!(f, "Program Association"),
            PIDTable::ConditionalAccess => write!(f, "Conditional Access"),
            PIDTable::TransportStreamDescription => write!(f, "Transport Stream Description"),
            PIDTable::IPMPControlInformation => write!(f, "IPMP Control Information"),
            PIDTable::AdaptiveStreamingInformation => write!(f, "Adaptive Streaming Information"),
            PIDTable::PID(val) => write!(f, "PID: {:#X}", val),
        }
    }
}

impl From<u16> for PIDTable {
    fn from(pid: u16) -> Self {
        match pid {
            0x1FFF => PIDTable::NullPacket,
            0x0000 => PIDTable::ProgramAssociation,
            0x0001 => PIDTable::ConditionalAccess,
            0x0002 => PIDTable::TransportStreamDescription,
            0x0003 => PIDTable::IPMPControlInformation,
            0x0004 => PIDTable::AdaptiveStreamingInformation,
            val => {
                if val > 0x000F {
                    PIDTable::PID(val)
                } else {
                    panic!("Unknown PID: {:#X}", val);
                }
            }
        }
    }
}

impl Into<u16> for PIDTable {
    fn into(self) -> u16 {
        match self {
            PIDTable::NullPacket => 0x1FFF,
            PIDTable::ProgramAssociation => 0x0000,
            PIDTable::ConditionalAccess => 0x0001,
            PIDTable::TransportStreamDescription => 0x0002,
            PIDTable::IPMPControlInformation => 0x0003,
            PIDTable::AdaptiveStreamingInformation => 0x0004,
            PIDTable::PID(val) => val,
        }
    }
}

impl Into<u16> for &PIDTable {
    fn into(self) -> u16 {
        match self {
            PIDTable::NullPacket => 0x1FFF,
            PIDTable::ProgramAssociation => 0x0000,
            PIDTable::ConditionalAccess => 0x0001,
            PIDTable::TransportStreamDescription => 0x0002,
            PIDTable::IPMPControlInformation => 0x0003,
            PIDTable::AdaptiveStreamingInformation => 0x0004,
            PIDTable::PID(val) => *val,
        }
    }
}
