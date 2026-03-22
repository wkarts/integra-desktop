use serde::{Deserialize, Serialize};

use super::{party::Party, taxes::Taxes};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessBatchInputItem {
    pub file_name: String,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessBatchResult {
    pub documents: Vec<NfseDocument>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldAction {
    Source,
    Zero,
    Empty,
    Ignore,
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRule {
    pub action: FieldAction,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionFieldRules {
    pub base_calculo: FieldRule,
    pub iss_aliquota: FieldRule,
    pub iss_devido: FieldRule,
    pub iss_retido: FieldRule,
    pub valor_iss: FieldRule,
    pub valor_liquido: FieldRule,
    pub valor_irrf: FieldRule,
    pub valor_inss: FieldRule,
    pub valor_pis: FieldRule,
    pub valor_cofins: FieldRule,
    pub valor_csll: FieldRule,
    pub descontos: FieldRule,
    pub deducoes: FieldRule,
    pub observacao: FieldRule,
    pub codigo_servico: FieldRule,
    pub municipio: FieldRule,
    pub serie: FieldRule,
    pub numero: FieldRule,
    pub data_emissao: FieldRule,
    pub data_competencia: FieldRule,
    pub tipo_documento: FieldRule,
    pub especie_documento: FieldRule,
    pub natureza_operacao: FieldRule,
    pub campos_complementares: FieldRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConversionProfile {
    pub profile_id: String,
    pub profile_name: String,
    pub profile_company_name: String,
    pub profile_company_document: String,
    pub user_company_name: String,
    pub user_company_document: String,
    pub output_layout: String,
    pub cod_prosoft: String,
    pub especie_documento: String,
    pub modelo_nf: String,
    pub tipo_documento: String,
    pub situacao_documento: String,
    pub cfps: String,
    pub cod_receita_irrf: String,
    pub cod_rec_pis: String,
    pub cod_rec_cofins: String,
    pub responsavel_retencao: String,
    pub tipo_recolhimento: String,
    pub motivo_retencao: String,
    pub operacao_nota: String,
    pub cst_pis: String,
    pub cst_cofins: String,
    pub cst_iss: String,
    pub company_municipio_nome: String,
    pub company_municipio_codigo: String,
    pub nfse_layout: String,
    pub obs_extended: String,
    pub field_rules: ConversionFieldRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileBundle {
    pub selected_profile_id: String,
    pub profiles: Vec<ConversionProfile>,
}

impl Default for ConversionProfile {
    fn default() -> Self {
        Self {
            profile_id: "empresa-padrao".into(),
            profile_name: "Empresa padrão".into(),
            profile_company_name: "Empresa padrão".into(),
            profile_company_document: String::new(),
            user_company_name: String::new(),
            user_company_document: String::new(),
            output_layout: "ba_prestados".into(),
            cod_prosoft: "0001".into(),
            especie_documento: "NFSE".into(),
            modelo_nf: "OU000".into(),
            tipo_documento: "001".into(),
            situacao_documento: String::new(),
            cfps: String::new(),
            cod_receita_irrf: String::new(),
            cod_rec_pis: String::new(),
            cod_rec_cofins: String::new(),
            responsavel_retencao: "0".into(),
            tipo_recolhimento: String::new(),
            motivo_retencao: String::new(),
            operacao_nota: String::new(),
            cst_pis: String::new(),
            cst_cofins: String::new(),
            cst_iss: String::new(),
            company_municipio_nome: String::new(),
            company_municipio_codigo: String::new(),
            nfse_layout: "auto".into(),
            obs_extended: "auto".into(),
            field_rules: ConversionFieldRules::default(),
        }
    }
}

impl Default for ProfileBundle {
    fn default() -> Self {
        let profile = ConversionProfile::default();
        Self {
            selected_profile_id: profile.profile_id.clone(),
            profiles: vec![profile],
        }
    }
}

impl Default for ConversionFieldRules {
    fn default() -> Self {
        let source = FieldRule {
            action: FieldAction::Source,
            value: None,
        };
        Self {
            base_calculo: source.clone(),
            iss_aliquota: source.clone(),
            iss_devido: source.clone(),
            iss_retido: source.clone(),
            valor_iss: source.clone(),
            valor_liquido: source.clone(),
            valor_irrf: source.clone(),
            valor_inss: source.clone(),
            valor_pis: source.clone(),
            valor_cofins: source.clone(),
            valor_csll: source.clone(),
            descontos: source.clone(),
            deducoes: source.clone(),
            observacao: source.clone(),
            codigo_servico: source.clone(),
            municipio: source.clone(),
            serie: source.clone(),
            numero: source.clone(),
            data_emissao: source.clone(),
            data_competencia: source.clone(),
            tipo_documento: source.clone(),
            especie_documento: source.clone(),
            natureza_operacao: source.clone(),
            campos_complementares: source,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfseDocument {
    pub id: String,
    pub file_name: String,
    pub provider: String,
    pub provider_friendly: String,
    pub layout: String,
    pub numero: String,
    pub serie: String,
    pub emissao: String,
    pub competencia: String,
    pub chave: String,
    pub municipio_codigo: String,
    pub municipio_nome: String,
    pub item_lista_servico: String,
    pub codigo_cnae: Option<String>,
    pub discriminacao: String,
    pub info_adic: String,
    pub prestador: Party,
    pub tomador: Party,
    pub taxes: Taxes,
    pub warnings: Vec<String>,
}
