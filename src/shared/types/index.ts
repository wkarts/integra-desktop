export type FieldAction = 'source' | 'zero' | 'empty' | 'ignore' | 'constant';

export interface FieldRule {
  action: FieldAction;
  value?: string;
}

export interface ConversionFieldRules {
  base_calculo: FieldRule;
  iss_aliquota: FieldRule;
  iss_devido: FieldRule;
  iss_retido: FieldRule;
  valor_iss: FieldRule;
  valor_liquido: FieldRule;
  valor_irrf: FieldRule;
  valor_inss: FieldRule;
  valor_pis: FieldRule;
  valor_cofins: FieldRule;
  valor_csll: FieldRule;
  descontos: FieldRule;
  deducoes: FieldRule;
  observacao: FieldRule;
  codigo_servico: FieldRule;
  municipio: FieldRule;
  serie: FieldRule;
  numero: FieldRule;
  data_emissao: FieldRule;
  data_competencia: FieldRule;
  tipo_documento: FieldRule;
  especie_documento: FieldRule;
  natureza_operacao: FieldRule;
  campos_complementares: FieldRule;
}

export type OutputLayout = 'ba_prestados' | 'ba_tomados' | 'prosoft_faturas';
export type NfseLayout = 'webiss_abrasf_v2' | 'ginfes' | 'betha' | 'abrasf_v1' | 'ubaira_custom' | 'auto';

export interface ConversionProfile {
  profile_id: string;
  profile_name: string;
  profile_company_name: string;
  profile_company_document: string;
  user_company_name: string;
  user_company_document: string;
  output_layout: OutputLayout;
  cod_prosoft: string;
  especie_documento: string;
  modelo_nf: string;
  tipo_documento: string;
  situacao_documento: string;
  cfps: string;
  cod_receita_irrf: string;
  cod_rec_pis: string;
  cod_rec_cofins: string;
  responsavel_retencao: string;
  tipo_recolhimento: string;
  motivo_retencao: string;
  operacao_nota: string;
  cst_pis: string;
  cst_cofins: string;
  cst_iss: string;
  company_municipio_nome: string;
  company_municipio_codigo: string;
  nfse_layout: NfseLayout;
  obs_extended: 'auto' | 'always' | 'never';
  field_rules: ConversionFieldRules;
}

export interface ProfileBundle {
  selected_profile_id: string;
  profiles: ConversionProfile[];
}

export interface LicenseSettings {
  service_url: string;
  company_name: string;
  company_document: string;
  company_email: string;
  station_name: string;
  machine_key: string;
  auto_register_machine: boolean;
  app_instance: string;
}

export interface LocalLicense {
  id: number;
  empresa: string;
  cnpj: string;
  fantasia: string;
  serial: string;
  licencas: string;
  ativo: boolean;
  endereco: string;
  bairro: string;
  cidade: string;
  uf: string;
  cep: string;
  numero: string;
  email: string;
  complemento: string;
  dias: number;
  competencia: string;
  bloqueio: boolean;
  retaguarda: boolean;
  pdv: boolean;
  cte: boolean;
  mdfe: boolean;
  nfe: boolean;
  frente: boolean;
  sat: boolean;
  app: boolean;
  boletos: boolean;
  mfe: boolean;
  commerce: boolean;
  serial_key: string;
  terminal_ativo: boolean;
  usadas: number;
}

export interface LicensedCompany {
  idcliente: number;
  datacad: string;
  cnpj: string;
  emp_inscestrg: string;
  emp_inscmunicipal: string;
  emp_nomefantasia: string;
  razaosocial: string;
  emp_endereco: string;
  emp_numero: string;
  emp_bairro: string;
  emp_cidade: string;
  emp_uf: string;
  emp_cep: string;
  emp_complemento: string;
  telefone1: string;
  emp_email: string;
  emp_website: string;
  emp_responsavel: string;
  emp_cnae: string;
  emp_serie: string;
  idrepresentante: number;
  bloqueio_admin: boolean;
  bloqueado: boolean;
  ativo: boolean;
  dia_venc_mensalidade: number;
  forma_pagamento: string;
  emp_obs: string;
  n_maquinas: number;
  data_val_lic: string;
  api_whatsapp: string;
  token_whatsapp: string;
  default_msg_whatsapp: string;
}

export interface LicensedDevice {
  idmaquina: number;
  cnpj: string;
  chave: string;
  nome: string;
  bloqueado: boolean;
  modulos: string;
  nome_compu: string;
  prog_acesso: string;
  cod_ace_remoto: string;
  versao_bd: string;
  versaoexe: string;
  sistema_operacional: string;
  memoria_ram: string;
  tipo: string;
  observacao: string;
  tecnico_instalacao: string;
  serial_number: string;
  hostname: string;
  station_name: string;
  machine_guid: string;
  bios_serial: string;
  motherboard_serial: string;
  full_device_name: string;
}

export interface LicenseRuntimeStatus {
  online: boolean;
  allowed: boolean;
  blocked: boolean;
  machine_registered: boolean;
  machine_blocked: boolean;
  seats_total: number;
  seats_used: number;
  expiry?: string | null;
  message: string;
  block_reason?: string | null;
  technical_message: string;
  company_name: string;
  company_document: string;
  machine_key: string;
  status_code: number;
  local_license?: LocalLicense | null;
  licensed_company?: LicensedCompany | null;
  licensed_device?: LicensedDevice | null;
}

export type LicenseCheckResult = LicenseRuntimeStatus;

export interface RegistrationDeviceInfo {
  station_name: string;
  device_display_name: string;
  hostname: string;
  computer_name: string;
  serial_number: string;
  machine_guid: string;
  bios_serial: string;
  motherboard_serial: string;
  logged_user: string;
  os_name: string;
  os_version: string;
  os_arch: string;
  domain_name: string;
  install_mode: string;
  mac_addresses: string[];
  device_key: string;
  registration_file_found: boolean;
  registration_file_path?: string | null;
  registration_file_verified?: boolean | null;
}

export interface AppMeta {
  product_name: string;
  version: string;
  build_hash: string;
  app_id: string;
}

export interface NfseParty {
  nome: string;
  documento: string;
  inscricao_municipal?: string;
  endereco?: string;
  municipio_codigo?: string;
  municipio_nome?: string;
  uf?: string;
  cep?: string;
}

export interface NfseTaxes {
  valor_servicos: number;
  base_calculo: number;
  valor_iss: number;
  aliquota_iss: number;
  valor_liquido: number;
  valor_irrf: number;
  valor_pis: number;
  valor_cofins: number;
  valor_csll: number;
  valor_inss: number;
  iss_retido: boolean;
}

export interface NfseDocument {
  id: string;
  file_name: string;
  provider: string;
  provider_friendly: string;
  layout: OutputLayout;
  numero: string;
  serie: string;
  emissao: string;
  competencia: string;
  chave: string;
  municipio_codigo: string;
  municipio_nome: string;
  item_lista_servico: string;
  codigo_cnae?: string;
  discriminacao: string;
  info_adic: string;
  prestador: NfseParty;
  tomador: NfseParty;
  taxes: NfseTaxes;
  warnings: string[];
}

export interface ProcessBatchInputItem {
  file_name: string;
  xml: string;
}

export interface ProcessBatchResult {
  documents: NfseDocument[];
  warnings: string[];
  errors: string[];
