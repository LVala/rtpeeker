use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceDescription {
    pub chunks: Vec<SourceDescriptionChunk>,
}

#[cfg(not(target_arch = "wasm32"))]
impl SourceDescription {
    pub fn new(packet: &rtcp::source_description::SourceDescription) -> Self {
        let chunks = packet
            .chunks
            .iter()
            .map(SourceDescriptionChunk::new)
            .collect();

        Self { chunks }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceDescriptionChunk {
    pub source: u32,
    pub items: Vec<SourceDescriptionItem>,
}

#[cfg(not(target_arch = "wasm32"))]
impl SourceDescriptionChunk {
    pub fn new(chunk: &rtcp::source_description::SourceDescriptionChunk) -> Self {
        let items = chunk.items.iter().map(SourceDescriptionItem::new).collect();

        Self {
            source: chunk.source,
            items,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceDescriptionItem {
    pub sdes_type: SdesType,
    pub text: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl SourceDescriptionItem {
    pub fn new(item: &rtcp::source_description::SourceDescriptionItem) -> Self {
        let text = std::str::from_utf8(&item.text[..]).unwrap().to_string();

        Self {
            sdes_type: item.sdes_type.into(),
            text,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SdesType {
    End,
    Cname,
    Name,
    Email,
    Phone,
    Location,
    Tool,
    Note,
    Private,
}

impl fmt::Display for SdesType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SdesType::*;

        let res = match self {
            End => "END",
            Cname => "CNAME",
            Name => "NAME",
            Email => "EMAIL",
            Phone => "PHONE",
            Location => "LOCATION",
            Tool => "TOOL",
            Note => "NOTE",
            Private => "PRIVATE",
        };

        write!(f, "{}", res)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<rtcp::source_description::SdesType> for SdesType {
    fn from(item: rtcp::source_description::SdesType) -> Self {
        use rtcp::source_description::SdesType::*;

        match item {
            SdesEnd => Self::End,
            SdesCname => Self::Cname,
            SdesName => Self::Name,
            SdesEmail => Self::Email,
            SdesPhone => Self::Phone,
            SdesLocation => Self::Location,
            SdesTool => Self::Tool,
            SdesNote => Self::Note,
            SdesPrivate => Self::Private,
        }
    }
}
