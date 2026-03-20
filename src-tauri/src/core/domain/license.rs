use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseSettings {
    pub service_url: String,
    pub company_name: String,
    pub company_document: String,
    pub company_email: String,
    pub station_name: String,
    pub machine_key: String,
    pub auto_register_machine: bool,
    pub app_instance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalLicense {
    pub id: i64,
    pub empresa: String,
    pub cnpj: String,
    pub fantasia: String,
    pub serial: String,
    pub licencas: String,
    pub ativo: bool,
    pub endereco: String,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub cep: String,
    pub numero: String,
    pub email: String,
    pub complemento: String,
    pub dias: i64,
    pub competencia: String,
    pub bloqueio: bool,
    pub retaguarda: bool,
    pub pdv: bool,
    pub cte: bool,
    pub mdfe: bool,
    pub nfe: bool,
    pub frente: bool,
    pub sat: bool,
    pub app: bool,
    pub boletos: bool,
    pub mfe: bool,
    pub commerce: bool,
    pub serial_key: String,
    pub terminal_ativo: bool,
    pub usadas: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicensedCompany {
    pub idcliente: i64,
    pub datacad: String,
    pub cnpj: String,
    pub emp_inscestrg: String,
    pub emp_inscmunicipal: String,
    pub emp_nomefantasia: String,
    pub razaosocial: String,
    pub emp_endereco: String,
    pub emp_numero: String,
    pub emp_bairro: String,
    pub emp_cidade: String,
    pub emp_uf: String,
    pub emp_cep: String,
    pub emp_complemento: String,
    pub telefone1: String,
    pub emp_email: String,
    pub emp_website: String,
    pub emp_responsavel: String,
    pub emp_cnae: String,
    pub emp_serie: String,
    pub idrepresentante: i64,
    pub bloqueio_admin: bool,
    pub bloqueado: bool,
    pub ativo: bool,
    pub dia_venc_mensalidade: i64,
    pub forma_pagamento: String,
    pub emp_obs: String,
    pub n_maquinas: i64,
    pub data_val_lic: String,
    pub api_whatsapp: String,
    pub token_whatsapp: String,
    pub default_msg_whatsapp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicensedDevice {
    pub idmaquina: i64,
    pub cnpj: String,
    pub chave: String,
    pub nome: String,
    pub bloqueado: bool,
    pub modulos: String,
    pub nome_compu: String,
    pub prog_acesso: String,
    pub cod_ace_remoto: String,
    pub versao_bd: String,
    pub versaoexe: String,
    pub sistema_operacional: String,
    pub memoria_ram: String,
    pub tipo: String,
    pub observacao: String,
    pub tecnico_instalacao: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseRuntimeStatus {
    pub online: bool,
    pub allowed: bool,
    pub blocked: bool,
    pub machine_registered: bool,
    pub machine_blocked: bool,
    pub seats_total: u32,
    pub seats_used: u32,
    pub expiry: Option<String>,
    pub message: String,
    pub block_reason: Option<String>,
    pub technical_message: String,
    pub company_name: String,
    pub company_document: String,
    pub machine_key: String,
    pub status_code: i32,
    pub local_license: Option<LocalLicense>,
    pub licensed_company: Option<LicensedCompany>,
    pub licensed_device: Option<LicensedDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseSnapshot {
    pub last_sync_at: String,
    pub local_license: Option<LocalLicense>,
    pub licensed_company: Option<LicensedCompany>,
    pub licensed_devices: Vec<LicensedDevice>,
    pub runtime_status: LicenseRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMeta {
    pub product_name: String,
    pub version: String,
    pub build_hash: String,
    pub app_id: String,
}
