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
  valor_liquido: FieldRule;
  valor_irrf: FieldRule;
  valor_inss: FieldRule;
  valor_pis: FieldRule;
  valor_cofins: FieldRule;
  valor_csll: FieldRule;
  observacao: FieldRule;
  codigo_servico: FieldRule;
}

export interface ConversionProfile {
  profile_name: string;
  output_layout: 'ba_prestados';
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
  layout: 'ba_prestados';
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
