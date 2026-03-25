pub mod abrasf_v1;
pub mod abrasf_v2;
pub mod betha;
pub mod detector;
pub mod generic_fallback;
pub mod ginfes;

use anyhow::{anyhow, Result};

use crate::core::domain::document::NfseDocument;

pub fn parse_nfse_xml(xml: &str, file_name: &str) -> Result<NfseDocument> {
    parse_nfse_xml_with_layout(xml, file_name, None)
}

pub fn parse_nfse_xml_with_layout(
    xml: &str,
    file_name: &str,
    configured_layout: Option<&str>,
) -> Result<NfseDocument> {
    let normalized_xml = normalize_xml_for_parsing(xml);
    let provider = configured_layout
        .and_then(provider_from_layout)
        .map(Ok)
        .unwrap_or_else(|| detector::detect_provider(&normalized_xml))?;

    match provider {
        detector::ProviderKind::WebissAbrasf202 | detector::ProviderKind::GenericCompNfse => {
            abrasf_v2::parse(&normalized_xml, file_name)
        }
        detector::ProviderKind::UbairaCustom => {
            generic_fallback::parse_ubaira(&normalized_xml, file_name)
        }
        detector::ProviderKind::Ginfes | detector::ProviderKind::Saj => {
            ginfes::parse(&normalized_xml, file_name)
        }
        detector::ProviderKind::Betha => betha::parse(&normalized_xml, file_name),
        detector::ProviderKind::AbrasfV1 => abrasf_v1::parse(&normalized_xml, file_name),
        detector::ProviderKind::Unknown => Err(anyhow!("Layout de XML NFS-e não suportado.")),
    }
}

fn normalize_xml_for_parsing(xml: &str) -> String {
    let trimmed = xml.trim_start_matches('\u{feff}').trim_start();

    if let Some(rest) = trimmed.strip_prefix("?<?xml") {
        return format!("<?xml{}", rest);
    }

    if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
        return trimmed.to_string();
    }

    if let Some(start) = trimmed.find('<') {
        return trimmed[start..].to_string();
    }

    trimmed.to_string()
}

fn provider_from_layout(layout: &str) -> Option<detector::ProviderKind> {
    match layout {
        "auto" | "" => None,
        "webiss_abrasf_v2" => Some(detector::ProviderKind::WebissAbrasf202),
        "ginfes" => Some(detector::ProviderKind::Ginfes),
        "betha" => Some(detector::ProviderKind::Betha),
        "abrasf_v1" => Some(detector::ProviderKind::AbrasfV1),
        "ubaira_custom" => Some(detector::ProviderKind::UbairaCustom),
        _ => None,
    }
}
