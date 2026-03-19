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

export interface LicenseCheckResult {
  allowed: boolean;
  online: boolean;
  active: boolean;
  blocked: boolean;
  device_registered: boolean;
  device_blocked: boolean;
  seats_total: number;
  seats_used: number;
  company_name: string;
  company_document: string;
  expires_at?: string | null;
  message: string;
  machine_key: string;
  status_code: number;
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
}

export interface UploadInputItem {
  file_name: string;
  kind: "xml" | "zip";
  content: string;
}
