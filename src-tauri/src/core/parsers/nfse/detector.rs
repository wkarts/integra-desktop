use anyhow::Result;
use roxmltree::Document;

#[derive(Debug, Clone, Copy)]
pub enum ProviderKind {
    WebissAbrasf202,
    UbairaCustom,
    GenericCompNfse,
    Ginfes,
    Betha,
    AbrasfV1,
    Unknown,
}

pub fn detect_provider(xml: &str) -> Result<ProviderKind> {
    let document = Document::parse(xml)?;
    let root = document.root_element();
    let root_name = root.tag_name().name();
    let namespace = root.tag_name().namespace().unwrap_or_default();

    if root_name == "CompNfse" && namespace.contains("abrasf") {
        return Ok(ProviderKind::WebissAbrasf202);
    }
    if root_name == "GerarNfseResposta" {
        return Ok(ProviderKind::UbairaCustom);
    }
    if root_name == "CompNfse" {
        return Ok(ProviderKind::GenericCompNfse);
    }
    if xml.contains("GINFES") {
        return Ok(ProviderKind::Ginfes);
    }
    if xml.contains("betha") {
        return Ok(ProviderKind::Betha);
    }
    Ok(ProviderKind::Unknown)
}
