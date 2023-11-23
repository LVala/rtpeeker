use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Packets,
    RtpPackets,
    RtcpPackets,
    Streams,
    Plot,
}

impl Tab {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Packets,
            Self::RtpPackets,
            Self::RtcpPackets,
            Self::Streams,
            Self::Plot,
        ]
    }

    pub fn from_string(tab_str: String) -> Option<Self> {
        Tab::all()
            .into_iter()
            .find(|tab| tab_str == tab.to_string())
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = match self {
            Self::Packets => "📦 All Packets",
            Self::RtpPackets => "🔈RTP Packets",
            Self::RtcpPackets => "📃 RTCP Packets",
            Self::Streams => "🔴 Streams",
            Self::Plot => "📈 Plot",
        };

        write!(f, "{}", ret)
    }
}
