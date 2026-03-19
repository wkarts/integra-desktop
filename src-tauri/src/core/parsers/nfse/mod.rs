pub mod abrasf_v1;
pub mod abrasf_v2;
pub mod betha;
pub mod detector;
pub mod generic_fallback;
pub mod ginfes;

use anyhow::{anyhow, Result};

use crate::core::domain::document::NfseDocument;

pub fn parse_nfse_xml(xml: &str, file_name: &str) -> Result<NfseDocument> {
    match detector::detect_provider(xml)? {
        detector::ProviderKind::WebissAbrasf202 | detector::ProviderKind::GenericCompNfse => {
            abrasf_v2::parse(xml, file_name)
        }
        detector::ProviderKind::UbairaCustom => generic_fallback::parse_ubaira(xml, file_name),
        detector::ProviderKind::Ginfes => ginfes::parse(xml, file_name),
        detector::ProviderKind::Betha => betha::parse(xml, file_name),
        detector::ProviderKind::AbrasfV1 => abrasf_v1::parse(xml, file_name),
        detector::ProviderKind::Unknown => Err(anyhow!("Layout de XML NFS-e não suportado.")),
    }
}
