use anyhow::Result;
use roxmltree::Document;

#[derive(Debug, Clone, Copy)]
pub enum ProviderKind {
    WebissAbrasf202,
    UbairaCustom,
    GenericCompNfse,
    Ginfes,
    Betha,
    Saj,
    AbrasfV1,
    Unknown,
}

pub fn detect_provider(xml: &str) -> Result<ProviderKind> {
    let document = Document::parse(xml)?;
    let root = document.root_element();
    let root_name = root.tag_name().name();
    let namespace = root
        .tag_name()
        .namespace()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let xml_lower = xml.to_ascii_lowercase();

    if root_name == "CompNfse" && namespace.contains("abrasf") {
        return Ok(ProviderKind::WebissAbrasf202);
    }
    if root_name == "GerarNfseResposta" || xml_lower.contains("ubaira") {
        return Ok(ProviderKind::UbairaCustom);
    }
    if xml_lower.contains("saj") {
        return Ok(ProviderKind::Saj);
    }
    if root_name == "CompNfse" {
        return Ok(ProviderKind::GenericCompNfse);
    }
    if xml_lower.contains("ginfes") {
        return Ok(ProviderKind::Ginfes);
    }
    if xml_lower.contains("betha") {
        return Ok(ProviderKind::Betha);
    }
    if xml_lower.contains("abrasf") {
        return Ok(ProviderKind::AbrasfV1);
    }

    Ok(ProviderKind::Unknown)
}
