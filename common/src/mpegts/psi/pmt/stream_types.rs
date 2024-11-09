use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone, Ord, PartialOrd)]
pub enum StreamTypes {
    ItuTIsoIecReserved,
    IsoIec111722Video,
    RecItuTH262OrIsoIec138182Video,
    IsoIec111723Audio,
    IsoIec138183Audio,
    RecItuTH2220OrIsoIec138181PrivateSections,
    RecItuTH2220OrIsoIec138181PESPackets,
    IsoIec13522MHEG,
    RecItuTH2220OrIsoIec138181AnnexADSMCC,
    RecItuTH2221,
    IsoIec138186TypeA,
    IsoIec138186TypeB,
    IsoIec138186TypeC,
    IsoIec138186TypeD,
    RecItuTH2220OrIsoIec138181Auxiliary,
    IsoIec138187AudioWithADTSTransportSyntax,
    IsoIec144962Visual,
    IsoIec144963AudioWithLATMTransportSyntax,
    IsoIec144961SLPacketizedStreamOrFlexMuxStreamInPESPackets,
    IsoIec144961SLPacketizedStreamOrFlexMuxStreamInIsoIec14496Sections,
    IsoIec138186SynchronizedDownloadProtocol,
    MetadataInPESPackets,
    MetadataInMetadataSections,
    MetadataInIsoIec138186DataCarousel,
    MetadataInIsoIec138186ObjectCarousel,
    MetadataInIsoIec138186SynchronizedDownloadProtocol,
    IpmpstreamDefinedInIsoIec1381811mpeg2,
    AVCVideoStreamAsDefinedInItuTH264OrIsoIec1449610Video,
    IsoIec144963AudioWithoutAdditionalTransportSyntax,
    IsoIec1449617Text,
    AuxiliaryVideoStreamAsDefinedInIsoIec230023,
    SVCVideoStreamAsDefinedInIsoIec1449610,
    MVCVideoStreamAsDefinedInIsoIec1449610,
    VideoStreamConformingToOneOrMoreProfilesAsDefinedInRecItuTT800OrIsoIec154441,
    AdditionalViewRecItuTH262OrIsoIec138182VideoStreamForServiceCompatibleStereoscopic3DServices,
    AdditionalViewRecItuTH264OrIsoIec1449610VideoStreamForServiceCompatible,
    RecItuTH265OrIsoIec230082VideoStreamOrAnHEVCTemporalVideoSubBitstream,
    HEVCTemporalVideoSubsetOfAnHEVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexAOfRecItuTH265OrIsoIec230082,
    MVCDVideoSubBitstreamOfAnAVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexIOfRecItuTH264OrIsoIec1449610,
    TimelineAndExternalMediaInformationStream,
    HEVCEnhancementSubPartitionWhichIncludesTemporalId0OfAnHEVCVideoStreamWhereAllNALsUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082,
    HEVCTemporalEnhancementSubPartitionOfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082,
    HEVCEnhancementSubPartitionWhichIncludesTemporalId0OfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexHOfRecItuTH265OrIsoIec230082,
    HEVCTemporalEnhancementSubPartitionOfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexHOfRecItuTH265OrIsoIec230082,
    GreenAccessUnitsCarriedInMPEG2Sections,
    IsoIec230083AudioWithMHASTransportSyntaxMainStream,
    IsoIec230083AudioWithMHASTransportSyntaxAuxiliaryStream,
    QualityAccessUnitsCarriedInSections,
    RecItuTH2220OrIsoIec138181Reserved,
    IPMPStream,
    UserPrivate,
}

impl From<u8> for StreamTypes {
    fn from(stream_type: u8) -> Self {
        match stream_type {
            0x00 => StreamTypes::ItuTIsoIecReserved,
            0x01 => StreamTypes::IsoIec111722Video,
            0x02 => StreamTypes::RecItuTH262OrIsoIec138182Video,
            0x03 => StreamTypes::IsoIec111723Audio,
            0x04 => StreamTypes::IsoIec138183Audio,
            0x05 => StreamTypes::RecItuTH2220OrIsoIec138181PrivateSections,
            0x06 => StreamTypes::RecItuTH2220OrIsoIec138181PESPackets,
            0x07 => StreamTypes::IsoIec13522MHEG,
            0x08 => StreamTypes::RecItuTH2220OrIsoIec138181AnnexADSMCC,
            0x09 => StreamTypes::RecItuTH2221,
            0x0A => StreamTypes::IsoIec138186TypeA,
            0x0B => StreamTypes::IsoIec138186TypeB,
            0x0C => StreamTypes::IsoIec138186TypeC,
            0x0D => StreamTypes::IsoIec138186TypeD,
            0x0E => StreamTypes::RecItuTH2220OrIsoIec138181Auxiliary,
            0x0F => StreamTypes::IsoIec138187AudioWithADTSTransportSyntax,
            0x10 => StreamTypes::IsoIec144962Visual,
            0x11 => StreamTypes::IsoIec144963AudioWithLATMTransportSyntax,
            0x12 => StreamTypes::IsoIec144961SLPacketizedStreamOrFlexMuxStreamInPESPackets,
            0x13 => StreamTypes::IsoIec144961SLPacketizedStreamOrFlexMuxStreamInIsoIec14496Sections,
            0x14 => StreamTypes::IsoIec138186SynchronizedDownloadProtocol,
            0x15 => StreamTypes::MetadataInPESPackets,
            0x16 => StreamTypes::MetadataInMetadataSections,
            0x17 => StreamTypes::MetadataInIsoIec138186DataCarousel,
            0x18 => StreamTypes::MetadataInIsoIec138186ObjectCarousel,
            0x19 => StreamTypes::MetadataInIsoIec138186SynchronizedDownloadProtocol,
            0x1A => StreamTypes::IpmpstreamDefinedInIsoIec1381811mpeg2,
            0x1B => StreamTypes::AVCVideoStreamAsDefinedInItuTH264OrIsoIec1449610Video,
            0x1C => StreamTypes::IsoIec144963AudioWithoutAdditionalTransportSyntax,
            0x1D => StreamTypes::IsoIec1449617Text,
            0x1E => StreamTypes::AuxiliaryVideoStreamAsDefinedInIsoIec230023,
            0x1F => StreamTypes::SVCVideoStreamAsDefinedInIsoIec1449610,
            0x20 => StreamTypes::MVCVideoStreamAsDefinedInIsoIec1449610,
            0x21 => StreamTypes::VideoStreamConformingToOneOrMoreProfilesAsDefinedInRecItuTT800OrIsoIec154441,
            0x22 => StreamTypes::AdditionalViewRecItuTH262OrIsoIec138182VideoStreamForServiceCompatibleStereoscopic3DServices,
            0x23 => StreamTypes::AdditionalViewRecItuTH264OrIsoIec1449610VideoStreamForServiceCompatible,
            0x24 => StreamTypes::RecItuTH265OrIsoIec230082VideoStreamOrAnHEVCTemporalVideoSubBitstream,
            0x25 => StreamTypes::HEVCTemporalVideoSubsetOfAnHEVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexAOfRecItuTH265OrIsoIec230082,
            0x26 => StreamTypes::MVCDVideoSubBitstreamOfAnAVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexIOfRecItuTH264OrIsoIec1449610,
            0x27 => StreamTypes::TimelineAndExternalMediaInformationStream,
            0x28 => StreamTypes::HEVCEnhancementSubPartitionWhichIncludesTemporalId0OfAnHEVCVideoStreamWhereAllNALsUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082,
            0x29 => StreamTypes::HEVCTemporalEnhancementSubPartitionOfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082,
            0x2A => StreamTypes::HEVCEnhancementSubPartitionWhichIncludesTemporalId0OfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexHOfRecItuTH265OrIsoIec230082,
            0x2B => StreamTypes::HEVCTemporalEnhancementSubPartitionOfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexHOfRecItuTH265OrIsoIec230082,
            0x2C => StreamTypes::GreenAccessUnitsCarriedInMPEG2Sections,
            0x2D => StreamTypes::IsoIec230083AudioWithMHASTransportSyntaxMainStream,
            0x2E => StreamTypes::IsoIec230083AudioWithMHASTransportSyntaxAuxiliaryStream,
            0x2F => StreamTypes::QualityAccessUnitsCarriedInSections,
            0x30..=0x7E => StreamTypes::RecItuTH2220OrIsoIec138181Reserved,
            0x7F => StreamTypes::IPMPStream,
            _ => StreamTypes::UserPrivate,
        }
    }
}

impl Display for StreamTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StreamTypes::ItuTIsoIecReserved => "ITU-T | ISO/IEC Reserved".to_string(),
            StreamTypes::IsoIec111722Video => "ISO/IEC 11172-2 Video".to_string(),
            StreamTypes::RecItuTH262OrIsoIec138182Video => "Rec. ITU-T H.262 | ISO/IEC 13818-2 Video".to_string(),
            StreamTypes::IsoIec111723Audio => "ISO/IEC 11172-3 Audio".to_string(),
            StreamTypes::IsoIec138183Audio => "ISO/IEC 13818-3 Audio".to_string(),
            StreamTypes::RecItuTH2220OrIsoIec138181PrivateSections => "Rec. ITU-T H.222.0 | ISO/IEC 13818-1 Private Sections".to_string(),
            StreamTypes::RecItuTH2220OrIsoIec138181PESPackets => "Rec. ITU-T H.222.0 | ISO/IEC 13818-1 PES Packets containing private data".to_string(),
            StreamTypes::IsoIec13522MHEG => "ISO/IEC 13522 MHEG".to_string(),
            StreamTypes::RecItuTH2220OrIsoIec138181AnnexADSMCC => "Rec. ITU-T H.222.0 | ISO/IEC 13818-1 Annex A DSM-CC".to_string(),
            StreamTypes::RecItuTH2221 => "Rec. ITU-T H.222.1".to_string(),
            StreamTypes::IsoIec138186TypeA => "ISO/IEC 13818-6 Type A".to_string(),
            StreamTypes::IsoIec138186TypeB => "ISO/IEC 13818-6 Type B".to_string(),
            StreamTypes::IsoIec138186TypeC => "ISO/IEC 13818-6 Type C".to_string(),
            StreamTypes::IsoIec138186TypeD => "ISO/IEC 13818-6 Type D".to_string(),
            StreamTypes::RecItuTH2220OrIsoIec138181Auxiliary => "Rec. ITU-T H.222.0 | ISO/IEC".to_string(),
            StreamTypes::IsoIec138187AudioWithADTSTransportSyntax => "ISO/IEC 13818-7 Audio with ADTS transport syntax".to_string(),
            StreamTypes::IsoIec144962Visual => "ISO/IEC 14496-2 Visual".to_string(),
            StreamTypes::IsoIec144963AudioWithLATMTransportSyntax => "ISO/IEC 14496-3 Audio with the LATM transport syntax as defined in ISO/IEC 14496-3".to_string(),
            StreamTypes::IsoIec144961SLPacketizedStreamOrFlexMuxStreamInPESPackets => "ISO/IEC 14496-1 SL-packetized stream or FlexMux stream carried in PES packets".to_string(),
            StreamTypes::IsoIec144961SLPacketizedStreamOrFlexMuxStreamInIsoIec14496Sections => "ISO/IEC 14496-1 SL-packetized stream or FlexMux stream carried in ISO/IEC 14496 sections".to_string(),
            StreamTypes::IsoIec138186SynchronizedDownloadProtocol => "ISO/IEC 13818-6 Synchronized Download Protocol".to_string(),
            StreamTypes::MetadataInPESPackets => "Metadata carried in PES packets".to_string(),
            StreamTypes::MetadataInMetadataSections => "Metadata carried in metadata sections".to_string(),
            StreamTypes::MetadataInIsoIec138186DataCarousel => "Metadata carried in ISO/IEC 13818-6 Data Carousel".to_string(),
            StreamTypes::MetadataInIsoIec138186ObjectCarousel => "Metadata carried in ISO/IEC 13818-6 Object Carousel".to_string(),
            StreamTypes::MetadataInIsoIec138186SynchronizedDownloadProtocol => "Metadata carried in ISO/IEC 13818-6 Synchronized Download Protocol".to_string(),
            StreamTypes::IpmpstreamDefinedInIsoIec1381811mpeg2 => "IPMP stream (defined in ISO/IEC 13818-11, MPEG-2 IPMP)".to_string(),
            StreamTypes::AVCVideoStreamAsDefinedInItuTH264OrIsoIec1449610Video => "AVC video stream conforming to one or more profiles defined in Annex A of Rec. ITU-T H.264 |
ISO/IEC 14496-10 or AVC video sub-bitstream of SVC as defined in 2.1.10 or MVC base view
sub-bitstream, as defined in 2.1.83, or AVC video sub-bitstream of MVC, as defined in 2.1.8 or
MVCD base view sub-bitstream, as defined in 2.1.88, or AVC video sub-bitstream of MVCD, as
defined in 2.1.9, or AVC base layer of an HEVC video stream conforming to one or more profiles
defined in Annex G or Annex H of Rec. ITU-T H.265 | ISO/IEC 23008-2".to_string(),
            StreamTypes::IsoIec144963AudioWithoutAdditionalTransportSyntax => "ISO/IEC 14496-3 Audio, without using any additional transport syntax, such as DST, ALS and SLS".to_string(),
            StreamTypes::IsoIec1449617Text => "ISO/IEC 14496-17 Text".to_string(),
            StreamTypes::AuxiliaryVideoStreamAsDefinedInIsoIec230023 => "Auxiliary video stream as defined in ISO/IEC 23002-3".to_string(),
            StreamTypes::SVCVideoStreamAsDefinedInIsoIec1449610 => "SVC video sub-bitstream of an AVC video stream conforming to one or more profiles defined in
Annex G of Rec. ITU-T H.264 | ISO/IEC 14496-10".to_string(),
            StreamTypes::MVCVideoStreamAsDefinedInIsoIec1449610 => "MVC video sub-bitstream of an AVC video stream conforming to one or more profiles defined in
Annex H of Rec. ITU-T H.264 | ISO/IEC 14496-10".to_string(),
            StreamTypes::VideoStreamConformingToOneOrMoreProfilesAsDefinedInRecItuTT800OrIsoIec154441 => "Video stream conforming to one or more profiles as defined in Rec. ITU-T T.800 | ISO/IEC 15444-1".to_string(),
            StreamTypes::AdditionalViewRecItuTH262OrIsoIec138182VideoStreamForServiceCompatibleStereoscopic3DServices => "Additional view Rec. ITU-T H.262 | ISO/IEC 13818-2 video stream for service-compatible
stereoscopic 3D services (see Notes 3 and 4)".to_string(),
            StreamTypes::AdditionalViewRecItuTH264OrIsoIec1449610VideoStreamForServiceCompatible => "Additional view Rec. ITU-T H.264 | ISO/IEC 14496-10 video stream conforming to one or more
profiles defined in Annex A for service-compatible stereoscopic 3D services (see Notes 3 and 4)".to_string(),
            StreamTypes::RecItuTH265OrIsoIec230082VideoStreamOrAnHEVCTemporalVideoSubBitstream => "Rec. ITU-T H.265 | ISO/IEC 23008-2 video stream or an HEVC temporal video sub-bitstream".to_string(),
            StreamTypes::HEVCTemporalVideoSubsetOfAnHEVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexAOfRecItuTH265OrIsoIec230082 => "HEVC temporal video subset of an HEVC video stream conforming to one or more profiles defined
in Annex A of Rec. ITU-T H.265 | ISO/IEC 23008-2".to_string(),
            StreamTypes::MVCDVideoSubBitstreamOfAnAVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexIOfRecItuTH264OrIsoIec1449610 => "MVCD video sub-bitstream of an AVC video stream conforming to one or more profiles defined in
Annex I of Rec. ITU-T H.264 | ISO/IEC 14496-10".to_string(),
            StreamTypes::TimelineAndExternalMediaInformationStream => "Timeline and external media information stream".to_string(),
            StreamTypes::HEVCEnhancementSubPartitionWhichIncludesTemporalId0OfAnHEVCVideoStreamWhereAllNALsUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082 => "HEVC enhancement sub-partition which includes temporal_id 0 of an HEVC video stream where all
NAL units contained in the stream conform to one or more profiles defined in Annex G of Rec. ITU-T H.265 | ISO/IEC 23008-2".to_string(),
            StreamTypes::HEVCTemporalEnhancementSubPartitionOfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082 => "HEVC temporal enhancement sub-partition of an HEVC video stream where all NAL units contained
in the stream conform to one or more profiles defined in Annex G of Rec. ITU-T H.265 | ISO/IEC 23008-2".to_string(),
            StreamTypes::GreenAccessUnitsCarriedInMPEG2Sections => "Green access units carried in MPEG-2 sections".to_string(),
            StreamTypes::IsoIec230083AudioWithMHASTransportSyntaxMainStream => "ISO/IEC 23008-3 audio with the MHA transport syntax, main audio stream".to_string(),
            StreamTypes::IsoIec230083AudioWithMHASTransportSyntaxAuxiliaryStream => "ISO/IEC 23008-3 audio with the MHA transport syntax, auxiliary audio stream".to_string(),
            StreamTypes::QualityAccessUnitsCarriedInSections => "Quality access units carried in sections".to_string(),
            StreamTypes::RecItuTH2220OrIsoIec138181Reserved => "Rec. ITU-T H.222.0 | ISO/IEC 13818-1 reserved".to_string(),
            StreamTypes::IPMPStream => "IPMP stream".to_string(),
            StreamTypes::UserPrivate => "User Private".to_string(),
            _ => "Unknown".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_types() {
        assert_eq!(StreamTypes::from(0x00), StreamTypes::ItuTIsoIecReserved);
        assert_eq!(StreamTypes::from(0x01), StreamTypes::IsoIec111722Video);
        assert_eq!(
            StreamTypes::from(0x02),
            StreamTypes::RecItuTH262OrIsoIec138182Video
        );
        assert_eq!(StreamTypes::from(0x03), StreamTypes::IsoIec111723Audio);
        assert_eq!(StreamTypes::from(0x04), StreamTypes::IsoIec138183Audio);
        assert_eq!(
            StreamTypes::from(0x05),
            StreamTypes::RecItuTH2220OrIsoIec138181PrivateSections
        );
        assert_eq!(
            StreamTypes::from(0x06),
            StreamTypes::RecItuTH2220OrIsoIec138181PESPackets
        );
        assert_eq!(StreamTypes::from(0x07), StreamTypes::IsoIec13522MHEG);
        assert_eq!(
            StreamTypes::from(0x08),
            StreamTypes::RecItuTH2220OrIsoIec138181AnnexADSMCC
        );
        assert_eq!(StreamTypes::from(0x09), StreamTypes::RecItuTH2221);
        assert_eq!(StreamTypes::from(0x0A), StreamTypes::IsoIec138186TypeA);
        assert_eq!(StreamTypes::from(0x0B), StreamTypes::IsoIec138186TypeB);
        assert_eq!(StreamTypes::from(0x0C), StreamTypes::IsoIec138186TypeC);
        assert_eq!(StreamTypes::from(0x0D), StreamTypes::IsoIec138186TypeD);
        assert_eq!(
            StreamTypes::from(0x0E),
            StreamTypes::RecItuTH2220OrIsoIec138181Auxiliary
        );
        assert_eq!(
            StreamTypes::from(0x0F),
            StreamTypes::IsoIec138187AudioWithADTSTransportSyntax
        );
        assert_eq!(StreamTypes::from(0x10), StreamTypes::IsoIec144962Visual);
        assert_eq!(
            StreamTypes::from(0x11),
            StreamTypes::IsoIec144963AudioWithLATMTransportSyntax
        );
        assert_eq!(
            StreamTypes::from(0x12),
            StreamTypes::IsoIec144961SLPacketizedStreamOrFlexMuxStreamInPESPackets
        );
        assert_eq!(
            StreamTypes::from(0x13),
            StreamTypes::IsoIec144961SLPacketizedStreamOrFlexMuxStreamInIsoIec14496Sections
        );
        assert_eq!(
            StreamTypes::from(0x14),
            StreamTypes::IsoIec138186SynchronizedDownloadProtocol
        );
        assert_eq!(StreamTypes::from(0x15), StreamTypes::MetadataInPESPackets);
        assert_eq!(
            StreamTypes::from(0x16),
            StreamTypes::MetadataInMetadataSections
        );
        assert_eq!(
            StreamTypes::from(0x17),
            StreamTypes::MetadataInIsoIec138186DataCarousel
        );
        assert_eq!(
            StreamTypes::from(0x18),
            StreamTypes::MetadataInIsoIec138186ObjectCarousel
        );
        assert_eq!(
            StreamTypes::from(0x19),
            StreamTypes::MetadataInIsoIec138186SynchronizedDownloadProtocol
        );
        assert_eq!(
            StreamTypes::from(0x1A),
            StreamTypes::IpmpstreamDefinedInIsoIec1381811mpeg2
        );
        assert_eq!(
            StreamTypes::from(0x1B),
            StreamTypes::AVCVideoStreamAsDefinedInItuTH264OrIsoIec1449610Video
        );
        assert_eq!(
            StreamTypes::from(0x1C),
            StreamTypes::IsoIec144963AudioWithoutAdditionalTransportSyntax
        );
        assert_eq!(StreamTypes::from(0x1D), StreamTypes::IsoIec1449617Text);
        assert_eq!(
            StreamTypes::from(0x1E),
            StreamTypes::AuxiliaryVideoStreamAsDefinedInIsoIec230023
        );
        assert_eq!(
            StreamTypes::from(0x1F),
            StreamTypes::SVCVideoStreamAsDefinedInIsoIec1449610
        );
        assert_eq!(
            StreamTypes::from(0x20),
            StreamTypes::MVCVideoStreamAsDefinedInIsoIec1449610
        );
        assert_eq!(StreamTypes::from(0x21), StreamTypes::VideoStreamConformingToOneOrMoreProfilesAsDefinedInRecItuTT800OrIsoIec154441);
        assert_eq!(StreamTypes::from(0x22), StreamTypes::AdditionalViewRecItuTH262OrIsoIec138182VideoStreamForServiceCompatibleStereoscopic3DServices);
        assert_eq!(
            StreamTypes::from(0x23),
            StreamTypes::AdditionalViewRecItuTH264OrIsoIec1449610VideoStreamForServiceCompatible
        );
        assert_eq!(
            StreamTypes::from(0x24),
            StreamTypes::RecItuTH265OrIsoIec230082VideoStreamOrAnHEVCTemporalVideoSubBitstream
        );
        assert_eq!(StreamTypes::from(0x25), StreamTypes::HEVCTemporalVideoSubsetOfAnHEVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexAOfRecItuTH265OrIsoIec230082);
        assert_eq!(StreamTypes::from(0x26), StreamTypes::MVCDVideoSubBitstreamOfAnAVCVideoStreamConformingToOneOrMoreProfilesDefinedInAnnexIOfRecItuTH264OrIsoIec1449610);
        assert_eq!(
            StreamTypes::from(0x27),
            StreamTypes::TimelineAndExternalMediaInformationStream
        );
        assert_eq!(StreamTypes::from(0x28), StreamTypes::HEVCEnhancementSubPartitionWhichIncludesTemporalId0OfAnHEVCVideoStreamWhereAllNALsUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082);
        assert_eq!(StreamTypes::from(0x29), StreamTypes::HEVCTemporalEnhancementSubPartitionOfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexGOfRecItuTH265OrIsoIec230082);
        assert_eq!(StreamTypes::from(0x2A), StreamTypes::HEVCEnhancementSubPartitionWhichIncludesTemporalId0OfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexHOfRecItuTH265OrIsoIec230082);
        assert_eq!(StreamTypes::from(0x2B), StreamTypes::HEVCTemporalEnhancementSubPartitionOfAnHEVCVideoStreamWhereAllNALUnitsContainedInTheStreamConformToOneOrMoreProfilesDefinedInAnnexHOfRecItuTH265OrIsoIec230082);
        assert_eq!(
            StreamTypes::from(0x2C),
            StreamTypes::GreenAccessUnitsCarriedInMPEG2Sections
        );
        assert_eq!(
            StreamTypes::from(0x2D),
            StreamTypes::IsoIec230083AudioWithMHASTransportSyntaxMainStream
        );
        assert_eq!(
            StreamTypes::from(0x2E),
            StreamTypes::IsoIec230083AudioWithMHASTransportSyntaxAuxiliaryStream
        );
        assert_eq!(
            StreamTypes::from(0x2F),
            StreamTypes::QualityAccessUnitsCarriedInSections
        );
        assert_eq!(
            StreamTypes::from(0x30),
            StreamTypes::RecItuTH2220OrIsoIec138181Reserved
        );
        assert_eq!(StreamTypes::from(0x7F), StreamTypes::IPMPStream);
        assert_eq!(StreamTypes::from(0x80), StreamTypes::UserPrivate);
    }

    #[test]
    fn test_display_for_stream_types() {
        assert_eq!(
            format!("{}", StreamTypes::ItuTIsoIecReserved),
            "ITU-T | ISO/IEC Reserved"
        );
        assert_eq!(
            format!("{}", StreamTypes::IsoIec111722Video),
            "ISO/IEC 11172-2 Video"
        );
        assert_eq!(
            format!("{}", StreamTypes::RecItuTH262OrIsoIec138182Video),
            "Rec. ITU-T H.262 | ISO/IEC 13818-2 Video"
        );
        assert_eq!(
            format!("{}", StreamTypes::IsoIec111723Audio),
            "ISO/IEC 11172-3 Audio"
        );
        assert_eq!(
            format!("{}", StreamTypes::IsoIec138183Audio),
            "ISO/IEC 13818-3 Audio"
        );
        // Add more assertions for other variants as needed
    }
}
