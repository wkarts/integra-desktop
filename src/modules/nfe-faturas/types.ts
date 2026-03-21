export interface NfeFaturasSettings {
  cod_prosoft: string;
  chk_append_d: boolean;
  chk_forcar_duas_linhas: boolean;
  chk_usar_sped: boolean;
  chk_ie_sped_padrao: boolean;
  chk_somente_com_sped: boolean;
  chk_incluir_sem_dup: boolean;
  chk_venc30: boolean;
  chk_incluir_faturas_sped: boolean;
  chk_recriar_c140_c141: boolean;
  sel_modo_parcelas: string;
  num_qtd_parcelas_geral: number;
  txt_regras_fornecedor_parcelas: string;
  num_venc_intervalo_dias: number;
  txt_venc_dias_por_parcela: string;
  sel_multi_sped_modo: string;
  sel_consolidacao_interna_nfe: string;
  chk_consolidar_cnpj: boolean;
  txt_cnpjs_consolidar: string;
  origem: string;
  tipo: string;
  serie_digits: string;
  ctx_nota: string;
  chk_exportar_filtrados: boolean;
}

export interface NfeFaturasRow {
  chave: string;
  desdob: number;
  cnpj_filial: string;
  cnpj_cpf: string;
  uf: string;
  ie: string;
  nf_serie: string;
  nf_numero: string;
  data_emissao: string;
  data_entrada: string;
  num_fatura: string;
  data_vencimento: string;
  valor_bruto_fat: string;
  source: string;
  sped_matched: boolean;
  legado: boolean;
  consolidated: boolean;
}

export interface NfeMetaDup {
  n_dup: string;
  d_venc: string;
  v_dup: string;
}

export interface NfeMeta {
  chave: string;
  n_fat: string;
  v_orig: string;
  v_liq: string;
  v_nf: string;
  dup_list: NfeMetaDup[];
  nf_numero: string;
  serie: string;
  cnpj_terceiro: string;
  dt_emi: string;
  source: string;
}

export interface NamedText {
  name: string;
  text: string;
}

export interface NfeFaturasCounts {
  xml: number;
  sped: number;
  zip: number;
}

export interface NfeFaturasLogItem {
  ts: string;
  level: 'info' | 'warn' | 'error' | string;
  msg: string;
  meta?: string | null;
}

export interface NfeFaturasProcessResult {
  rows: NfeFaturasRow[];
  counts: NfeFaturasCounts;
  logs: NfeFaturasLogItem[];
  sped_files: NamedText[];
  nfe_metas: NfeMeta[];
}

export interface NfeFaturasLegacyPreview {
  notas: number;
  parcelas: number;
  invalid_count: number;
  invalid_lines: string[];
  warning_count: number;
  warnings: string[];
  divergences: string[];
  divergence_count: number;
}

export interface NfeFaturasLegacyResult {
  rows: NfeFaturasRow[];
  preview: NfeFaturasLegacyPreview;
  logs: NfeFaturasLogItem[];
}

export interface NfeFaturasExportResult {
  output_paths: string[];
  lines: number;
  records: number;
  message: string;
}

export const defaultNfeFaturasSettings: NfeFaturasSettings = {
  cod_prosoft: '',
  chk_append_d: false,
  chk_forcar_duas_linhas: false,
  chk_usar_sped: true,
  chk_ie_sped_padrao: false,
  chk_somente_com_sped: true,
  chk_incluir_sem_dup: false,
  chk_venc30: true,
  chk_incluir_faturas_sped: false,
  chk_recriar_c140_c141: false,
  sel_modo_parcelas: 'respeitar_xml',
  num_qtd_parcelas_geral: 1,
  txt_regras_fornecedor_parcelas: '',
  num_venc_intervalo_dias: 30,
  txt_venc_dias_por_parcela: '',
  sel_multi_sped_modo: 'atualizar_individual',
  sel_consolidacao_interna_nfe: 'nao',
  chk_consolidar_cnpj: false,
  txt_cnpjs_consolidar: '',
  origem: '0',
  tipo: '1',
  serie_digits: '0',
  ctx_nota: 'entrada',
  chk_exportar_filtrados: false,
};
