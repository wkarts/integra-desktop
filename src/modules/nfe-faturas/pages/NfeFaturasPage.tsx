import { useEffect, useMemo, useRef, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import './NfeFaturasPage.css';
import type {
  NamedText,
  NfeMeta,
  NfeFaturasLegacyPreview,
  NfeFaturasLogItem,
  NfeFaturasRow,
  NfeFaturasSettings,
} from '../types';
import { defaultNfeFaturasSettings } from '../types';
import {
  clipboardWriteText,
  dialogConfirm,
  dialogMessageError,
  dialogMessageInfo,
  dialogMessageWarning,
  dialogPickNfeFaturasDirectory,
  dialogPickNfeFaturasFiles,
  dialogPickNfeFaturasLegacyFile,
  dialogPickNfeFaturasOutputDir,
  dialogSaveNfeFaturasFile,
  exportNfeFaturasCsv,
  exportNfeFaturasSped,
  exportNfeFaturasTxt,
  guessNfeFaturasCnpjFilial,
  importNfeFaturasLegacy,
  loadNfeFaturasSettings,
  processNfeFaturasSelection,
  saveNfeFaturasSettings,
} from '../services/tauriService';

const emptyLegacyPreview: NfeFaturasLegacyPreview = {
  notas: 0,
  parcelas: 0,
  invalid_count: 0,
  invalid_lines: [],
  warning_count: 0,
  warnings: [],
  divergences: [],
  divergence_count: 0,
};

function onlyDigits(value: string): string {
  return (value ?? '').replace(/\D+/g, '');
}

function onlyAlphaNumUpper(value: string): string {
  return (value ?? '').toUpperCase().replace(/[^0-9A-Z]+/g, '');
}

function toMoney2(value: string): string {
  const source = `${value ?? ''}`.trim();
  if (!source) return '0.00';
  let normalized = source.replace(/\s+/g, '');
  if (normalized.includes(',') && normalized.includes('.')) {
    normalized = normalized.replace(/\./g, '').replace(',', '.');
  } else if (normalized.includes(',')) {
    normalized = normalized.replace(',', '.');
  }
  const n = Number(normalized);
  return Number.isFinite(n) ? n.toFixed(2) : '0.00';
}

function computeDestino(settings: NfeFaturasSettings): string {
  const raw = settings.cod_prosoft.trim();
  if (!raw) return '';
  let code = onlyDigits(raw) || raw;
  const hasD = /d$/i.test(raw) || settings.chk_append_d;
  if (hasD && !/d$/i.test(code)) code += 'D';
  if (!hasD && /d$/i.test(code)) code = code.slice(0, -1);
  return `Faturas${code}.txt`;
}

function mergeLogs(current: NfeFaturasLogItem[], next: NfeFaturasLogItem[]): NfeFaturasLogItem[] {
  return [...next, ...current].slice(0, 500);
}

export default function NfeFaturasPage() {
  const [settings, setSettings] = useState<NfeFaturasSettings>(defaultNfeFaturasSettings);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [rows, setRows] = useState<NfeFaturasRow[]>([]);
  const [logs, setLogs] = useState<NfeFaturasLogItem[]>([]);
  const [status, setStatus] = useState('Aguardando seleção…');
  const [progress, setProgress] = useState(0);
  const [processing, setProcessing] = useState(false);
  const [gridFilter, setGridFilter] = useState('');
  const [menuOpen, setMenuOpen] = useState(false);
  const [manualOpen, setManualOpen] = useState(false);
  const [cfgCollapsed, setCfgCollapsed] = useState(false);
  const [legacyFilePath, setLegacyFilePath] = useState('');
  const [legacyConferir, setLegacyConferir] = useState(false);
  const [legacyCnpjFilial, setLegacyCnpjFilial] = useState('');
  const [legacyPreview, setLegacyPreview] = useState<NfeFaturasLegacyPreview>(emptyLegacyPreview);
  const [counts, setCounts] = useState({ xml: 0, sped: 0, zip: 0 });
  const [spedFiles, setSpedFiles] = useState<NamedText[]>([]);
  const [nfeMetas, setNfeMetas] = useState<NfeMeta[]>([]);
  const saveTimer = useRef<number | null>(null);
  const loadedSettings = useRef(false);

  useEffect(() => {
    loadNfeFaturasSettings()
      .then((stored) => {
        setSettings(stored);
        loadedSettings.current = true;
      })
      .catch(() => {
        loadedSettings.current = true;
      });
  }, []);

  useEffect(() => {
    if (!loadedSettings.current) return;
    if (saveTimer.current) window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => {
      void saveNfeFaturasSettings(settings).catch(() => undefined);
    }, 250);
    return () => {
      if (saveTimer.current) window.clearTimeout(saveTimer.current);
    };
  }, [settings]);

  const destino = useMemo(() => computeDestino(settings), [settings]);

  const visibleRows = useMemo(() => {
    const q = gridFilter.trim().toLowerCase();
    if (!q) return rows;
    return rows.filter((row) => {
      const hay = [
        row.chave,
        row.cnpj_cpf,
        row.cnpj_filial,
        row.nf_numero,
        row.nf_serie,
        row.num_fatura,
        row.uf,
        row.ie,
        row.source,
      ]
        .join(' | ')
        .toLowerCase();
      return hay.includes(q);
    });
  }, [gridFilter, rows]);

  const stats = useMemo(() => {
    const list = visibleRows;
    const sum = list.reduce((acc, row) => acc + Number(toMoney2(row.valor_bruto_fat)), 0);
    const spedMatch = list.filter((row) => row.sped_matched).length;
    return {
      count: list.length,
      total: rows.length,
      sum: sum.toLocaleString('pt-BR', { minimumFractionDigits: 2, maximumFractionDigits: 2 }),
      spedMatch,
      spedMiss: list.length - spedMatch,
    };
  }, [rows.length, visibleRows]);

  const validation = useMemo(() => {
    if (!rows.length) return [] as { tone: 'ok' | 'warn'; text: string }[];
    let badCnpj = 0;
    let badUf = 0;
    let badChave = 0;
    for (const row of rows) {
      const c = onlyDigits(row.cnpj_cpf);
      if (c && c.length !== 11 && c.length !== 14) badCnpj += 1;
      if (row.uf && row.uf.trim().length !== 2) badUf += 1;
      const ch = onlyDigits(row.chave);
      if (ch && ch.length !== 44) badChave += 1;
    }
    const items: { tone: 'ok' | 'warn'; text: string }[] = [];
    if (badCnpj) items.push({ tone: 'warn', text: `CNPJ/CPF inválido: ${badCnpj}` });
    if (badUf) items.push({ tone: 'warn', text: `UF inválida: ${badUf}` });
    if (badChave) items.push({ tone: 'warn', text: `Chave inválida (≠ 44 dígitos): ${badChave}` });
    if (!badCnpj && !badUf && !badChave) items.push({ tone: 'ok', text: 'Validação básica OK' });
    if (settings.chk_usar_sped) {
      items.push({
        tone: spedFiles.length ? 'ok' : 'warn',
        text: spedFiles.length ? `SPED: MATCH ${rows.filter((row) => row.sped_matched).length} • MISSING ${rows.filter((row) => !row.sped_matched).length}` : 'SPED não carregado / não encontrado',
      });
    }
    if (rows.length && settings.chk_ie_sped_padrao) {
      items.push({ tone: 'ok', text: 'IE 0150 ativa em modo padrão SPED' });
    }
    return items;
  }, [rows, settings.chk_ie_sped_padrao, settings.chk_usar_sped, spedFiles.length]);

  function updateSetting<K extends keyof NfeFaturasSettings>(key: K, value: NfeFaturasSettings[K]) {
    setSettings((current) => ({ ...current, [key]: value }));
  }

  function appendStatus(text: string) {
    setStatus(text);
  }

  function resetProcessData() {
    setRows([]);
    setSpedFiles([]);
    setNfeMetas([]);
    setLegacyPreview(emptyLegacyPreview);
    setProgress(0);
    setStatus('Aguardando seleção…');
  }

  function clearAll() {
    setSelectedPaths([]);
    setCounts({ xml: 0, sped: 0, zip: 0 });
    resetProcessData();
    setLogs([]);
    setLegacyFilePath('');
    setLegacyCnpjFilial('');
  }

  function updateSelectedPaths(next: string[]) {
    const unique = Array.from(new Set(next.filter(Boolean)));
    setSelectedPaths(unique);
    const lower = unique.map((item) => item.toLowerCase());
    setCounts({
      xml: lower.filter((item) => item.endsWith('.xml')).length,
      sped: lower.filter((item) => item.endsWith('.txt') || item.endsWith('.sped')).length,
      zip: lower.filter((item) => item.endsWith('.zip')).length,
    });
  }

  async function handlePickFiles() {
    setMenuOpen(false);
    const result = await dialogPickNfeFaturasFiles();
    if (!result?.length) return;
    updateSelectedPaths([...(selectedPaths || []), ...result]);
    appendStatus('Entrada adicionada.');
  }

  async function handlePickDir() {
    setMenuOpen(false);
    const result = await dialogPickNfeFaturasDirectory();
    if (!result) return;
    updateSelectedPaths([...(selectedPaths || []), result]);
    appendStatus('Diretório adicionado.');
  }

  async function handleProcess() {
    if (!selectedPaths.length) {
      await dialogMessageWarning('Integra', 'Selecione arquivos ou uma pasta antes de processar.');
      return;
    }
    setProcessing(true);
    setProgress(10);
    appendStatus('Preparando entradas…');
    try {
      const result = await processNfeFaturasSelection(selectedPaths, settings);
      setProgress(100);
      setRows(result.rows);
      setSpedFiles(result.sped_files);
      setNfeMetas(result.nfe_metas);
      setCounts(result.counts);
      setLogs(result.logs);
      appendStatus(result.rows.length ? `Processado. Registros: ${result.rows.length}` : 'Nenhum registro gerado.');
    } catch (error) {
      appendStatus('Erro no processamento.');
      await dialogMessageError('Integra', `${error}`);
    } finally {
      setProcessing(false);
    }
  }

  async function handlePickLegacyFile() {
    const result = await dialogPickNfeFaturasLegacyFile();
    if (!result) return;
    setLegacyFilePath(result);
  }

  async function handleImportLegacy() {
    if (!legacyFilePath) {
      await dialogMessageWarning('Integra', 'Selecione o arquivo legado TXT/CSV.');
      return;
    }
    setProcessing(true);
    appendStatus('Importando legado…');
    try {
      const result = await importNfeFaturasLegacy(
        legacyFilePath,
        legacyCnpjFilial || null,
        legacyConferir,
        rows,
        spedFiles,
        nfeMetas,
      );
      setRows((current) => [...current.filter((item) => !item.legado), ...result.rows]);
      setLegacyPreview(result.preview);
      setLogs((current) => mergeLogs(current, result.logs));
      appendStatus(result.rows.length ? `Legado importado: ${result.rows.length} parcela(s).` : 'Legado sem registros válidos.');
    } catch (error) {
      await dialogMessageError('Integra', `${error}`);
    } finally {
      setProcessing(false);
    }
  }

  function handleClearLegacy() {
    setRows((current) => current.filter((item) => !item.legado));
    setLegacyPreview(emptyLegacyPreview);
    setLegacyFilePath('');
    appendStatus('Importação legado removida.');
  }

  async function handleLegacyAutoCnpj() {
    try {
      const result = await guessNfeFaturasCnpjFilial(rows, spedFiles);
      if (result) {
        setLegacyCnpjFilial(result);
        appendStatus('CNPJ da filial preenchido automaticamente.');
      } else {
        await dialogMessageWarning('Integra', 'Não foi possível identificar o CNPJ da filial na entrada atual.');
      }
    } catch (error) {
      await dialogMessageError('Integra', `${error}`);
    }
  }

  async function handleCopyLog() {
    const text = logs
      .slice()
      .reverse()
      .map((item) => `[${item.ts}] ${item.level.toUpperCase()} - ${item.msg}${item.meta ? ` | ${item.meta}` : ''}`)
      .join('\n');
    await clipboardWriteText(text || '');
    appendStatus('Log copiado para a área de transferência.');
  }

  async function handleExportTxt() {
    const path = await dialogSaveNfeFaturasFile(destino || 'Faturas.txt', 'Exportar TXT', ['txt']);
    if (!path) return;
    try {
      const result = await exportNfeFaturasTxt(settings.chk_exportar_filtrados ? visibleRows : rows, settings, path);
      appendStatus(result.message);
      await dialogMessageInfo('Integra', result.message);
    } catch (error) {
      await dialogMessageError('Integra', `${error}`);
    }
  }

  async function handleExportCsv() {
    const path = await dialogSaveNfeFaturasFile((destino || 'Faturas.txt').replace(/\.txt$/i, '.csv'), 'Exportar CSV', ['csv']);
    if (!path) return;
    try {
      const result = await exportNfeFaturasCsv(settings.chk_exportar_filtrados ? visibleRows : rows, settings, path);
      appendStatus(result.message);
      await dialogMessageInfo('Integra', result.message);
    } catch (error) {
      await dialogMessageError('Integra', `${error}`);
    }
  }

  async function handleExportSped() {
    const directory = await dialogPickNfeFaturasOutputDir();
    if (!directory) return;
    try {
      const result = await exportNfeFaturasSped(settings.chk_exportar_filtrados ? visibleRows : rows, settings, spedFiles, nfeMetas, directory);
      appendStatus(result.message);
      await dialogMessageInfo('Integra', result.message);
    } catch (error) {
      await dialogMessageError('Integra', `${error}`);
    }
  }

  async function handleClearAll() {
    if (rows.length || selectedPaths.length) {
      const accepted = await dialogConfirm('Integra', 'Deseja limpar entradas, grid e logs da tela?');
      if (!accepted) return;
    }
    clearAll();
  }

  function handleRowChange(index: number, key: keyof NfeFaturasRow, value: string) {
    setRows((current) =>
      current.map((row, idx) => {
        if (idx !== index) return row;
        let nextValue = value;
        if (key === 'cnpj_filial' || key === 'cnpj_cpf') nextValue = onlyDigits(value);
        if (key === 'uf') nextValue = value.toUpperCase().slice(0, 2);
        if (key === 'ie') nextValue = onlyAlphaNumUpper(value);
        if (key === 'valor_bruto_fat') nextValue = toMoney2(value);
        if (key === 'desdob') {
          return { ...row, desdob: onlyDigits(value) === '1' ? 1 : 0 };
        }
        if (key === 'chave') nextValue = onlyDigits(value).slice(-44);
        return { ...row, [key]: nextValue };
      }),
    );
  }

  return (
    <div className="nfe-faturas-page stack-lg">
      <PageHeader
        title="NFe / Faturas"
        subtitle="Fluxo migrado para Tauri nativo: seleção via dialog do app, processamento/exportação em Rust e persistência no storage do aplicativo."
        actions={
          <>
            <button className="btn" type="button" onClick={() => setManualOpen(true)}>Manual</button>
            <span className="badge">Desktop local</span>
          </>
        }
      />

      <div className="nfe-layout-top">
        <section className="card nfe-card">
          <div className="nfe-card-head">
            <strong>Entrada</strong>
          </div>

          <div className="nfe-drop" role="button" tabIndex={0} onClick={() => setMenuOpen((v) => !v)}>
            <div>
              <div className="nfe-drop-title">Selecionar arquivos ou pasta</div>
              <div className="muted">Tauri nativo: diálogo de abrir do sistema operacional.</div>
              <div className="nfe-chip-row">
                <span className="badge">XML</span>
                <span className="badge">SPED</span>
                <span className="badge">ZIP</span>
                <span className="badge">Pasta</span>
              </div>
            </div>
            <div className="nfe-counts">
              <span className="badge">XML: {counts.xml}</span>
              <span className="badge">SPED: {counts.sped}</span>
              <span className="badge">ZIP: {counts.zip}</span>
            </div>
          </div>

          {menuOpen ? (
            <div className="nfe-menu-panel">
              <button className="btn" type="button" onClick={() => void handlePickFiles()}>Arquivos (XML / SPED / ZIP)</button>
              <button className="btn" type="button" onClick={() => void handlePickDir()}>Pasta (com subpastas)</button>
              <button className="btn danger" type="button" onClick={() => setMenuOpen(false)}>Fechar</button>
            </div>
          ) : null}

          <div className="nfe-selection-list">
            {selectedPaths.length ? selectedPaths.map((path) => <div key={path} className="nfe-selection-item">{path}</div>) : <div className="muted">Nenhuma entrada selecionada.</div>}
          </div>

          <div className="actions-row">
            <button className="btn primary" type="button" disabled={processing || !selectedPaths.length} onClick={() => void handleProcess()}>Processar</button>
            <button className="btn danger" type="button" disabled={processing && progress < 100 ? false : false} onClick={() => void handleClearAll()}>Limpar</button>
          </div>

          <div className="nfe-progress-wrap">
            <progress max={100} value={progress} />
            <div className="muted">{status}</div>
          </div>

          <details className="nfe-box" open>
            <summary>Log de processamento</summary>
            <div className="actions-row nfe-log-actions">
              <button className="btn" type="button" onClick={() => void handleCopyLog()}>Copiar log</button>
              <button className="btn danger" type="button" onClick={() => setLogs([])}>Limpar log</button>
            </div>
            <div className="nfe-log-list">
              {logs.length ? logs.map((item, index) => (
                <div key={`${item.ts}-${index}`} className={`nfe-log-item ${item.level}`}>
                  <strong>{item.level.toUpperCase()}</strong>
                  <div>
                    <div>{item.msg}</div>
                    {item.meta ? <small>{item.meta}</small> : null}
                  </div>
                </div>
              )) : <div className="muted">Sem eventos ainda.</div>}
            </div>
          </details>
        </section>

        <section className="card nfe-card nfe-sticky-card">
          <div className="nfe-card-head between">
            <strong>Configurações e Saída</strong>
            <button className="btn" type="button" onClick={() => setCfgCollapsed((v) => !v)}>{cfgCollapsed ? 'Expandir tudo' : 'Recolher tudo'}</button>
          </div>

          <div className="form-grid cols-4 nfe-top-fields">
            <div>
              <label>Código Prosoft</label>
              <input value={settings.cod_prosoft} onChange={(e) => updateSetting('cod_prosoft', e.target.value)} />
            </div>
            <div>
              <label>Arquivo de saída</label>
              <input value={destino} readOnly />
            </div>
            <div>
              <label>Origem</label>
              <input value={settings.origem} maxLength={1} onChange={(e) => updateSetting('origem', e.target.value)} />
            </div>
            <div>
              <label>Tipo</label>
              <input value={settings.tipo} maxLength={1} onChange={(e) => updateSetting('tipo', e.target.value)} />
            </div>
          </div>

          <div className="nfe-validation-row">
            {validation.map((item, index) => <span key={`${item.text}-${index}`} className={`nfe-pill ${item.tone}`}>{item.text}</span>)}
          </div>

          <details className="nfe-box" open={!cfgCollapsed}>
            <summary>Exportação</summary>
            <div className="actions-row">
              <button className="btn primary" type="button" disabled={!rows.length || processing} onClick={() => void handleExportTxt()}>Exportar TXT</button>
              <button className="btn" type="button" disabled={!rows.length || processing} onClick={() => void handleExportCsv()}>Exportar CSV</button>
            </div>
          </details>

          <details className="nfe-box" open={!cfgCollapsed}>
            <summary>Exportação SPED (C140/C141)</summary>
            <div className="actions-row">
              <button className="btn success" type="button" disabled={!rows.length || !spedFiles.length || processing} onClick={() => void handleExportSped()}>Exportar SPED Atualizado</button>
            </div>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_incluir_faturas_sped} onChange={(e) => updateSetting('chk_incluir_faturas_sped', e.target.checked)} /> Incluir faturas no SPED</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_recriar_c140_c141} onChange={(e) => updateSetting('chk_recriar_c140_c141', e.target.checked)} /> Recriar C140/C141 se já existir</label>
            <div className="grid-two">
              <div>
                <label>Modo de parcelas</label>
                <select value={settings.sel_modo_parcelas} onChange={(e) => updateSetting('sel_modo_parcelas', e.target.value)}>
                  <option value="respeitar_xml">Respeitar XML</option>
                  <option value="forcar_geral">Forçar geral</option>
                  <option value="forcar_por_fornecedor">Forçar por fornecedor</option>
                </select>
              </div>
              <div>
                <label>Qtd. parcelas (geral)</label>
                <input type="number" min={1} value={settings.num_qtd_parcelas_geral} onChange={(e) => updateSetting('num_qtd_parcelas_geral', Number(e.target.value || 1))} />
              </div>
            </div>
            <div>
              <label>Regras por fornecedor (CNPJ=NUM_PARC)</label>
              <textarea value={settings.txt_regras_fornecedor_parcelas} onChange={(e) => updateSetting('txt_regras_fornecedor_parcelas', e.target.value)} />
            </div>
            <div className="grid-two">
              <div>
                <label>Intervalo padrão (dias)</label>
                <input type="number" value={settings.num_venc_intervalo_dias} onChange={(e) => updateSetting('num_venc_intervalo_dias', Number(e.target.value || 0))} />
              </div>
              <div>
                <label>Dias por parcela</label>
                <input value={settings.txt_venc_dias_por_parcela} onChange={(e) => updateSetting('txt_venc_dias_por_parcela', e.target.value)} />
              </div>
            </div>
            <div className="grid-two">
              <div>
                <label>Multi-SPED</label>
                <select value={settings.sel_multi_sped_modo} onChange={(e) => updateSetting('sel_multi_sped_modo', e.target.value)}>
                  <option value="atualizar_individual">Atualizar individual</option>
                  <option value="consolidar">Consolidar</option>
                  <option value="misto">Misto</option>
                </select>
              </div>
              <div>
                <label>Consolidação interna por NF-e</label>
                <select value={settings.sel_consolidacao_interna_nfe} onChange={(e) => updateSetting('sel_consolidacao_interna_nfe', e.target.value)}>
                  <option value="nao">Não consolidar</option>
                  <option value="reduzir_para_1_parcela">Reduzir para 1 parcela</option>
                  <option value="agrupar_por_data">Agrupar por data</option>
                </select>
              </div>
            </div>
          </details>

          <details className="nfe-box" open={!cfgCollapsed}>
            <summary>Opções</summary>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_append_d} onChange={(e) => updateSetting('chk_append_d', e.target.checked)} /> Anexar D no código</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_forcar_duas_linhas} onChange={(e) => updateSetting('chk_forcar_duas_linhas', e.target.checked)} /> Forçar 2 linhas por registro</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_usar_sped} onChange={(e) => updateSetting('chk_usar_sped', e.target.checked)} /> Usar SPED para escrituração e parcelas</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_ie_sped_padrao} onChange={(e) => updateSetting('chk_ie_sped_padrao', e.target.checked)} /> Considerar IE do SPED como padrão</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_somente_com_sped} onChange={(e) => updateSetting('chk_somente_com_sped', e.target.checked)} /> Exportar somente com SPED correspondente</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_incluir_sem_dup} onChange={(e) => updateSetting('chk_incluir_sem_dup', e.target.checked)} /> Incluir notas sem parcelamento</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_venc30} onChange={(e) => updateSetting('chk_venc30', e.target.checked)} /> Vencimento padrão +30 dias</label>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_exportar_filtrados} onChange={(e) => updateSetting('chk_exportar_filtrados', e.target.checked)} /> Exportar somente filtrados</label>
          </details>

          <details className="nfe-box" open={!cfgCollapsed}>
            <summary>Consolidar parcelas por CNPJ/CPF</summary>
            <label className="nfe-check"><input type="checkbox" checked={settings.chk_consolidar_cnpj} onChange={(e) => updateSetting('chk_consolidar_cnpj', e.target.checked)} /> Consolidar quando o documento estiver na lista</label>
            <div>
              <label>Lista de CNPJs/CPFs</label>
              <textarea value={settings.txt_cnpjs_consolidar} onChange={(e) => updateSetting('txt_cnpjs_consolidar', e.target.value)} />
            </div>
          </details>

          <details className="nfe-box" open={!cfgCollapsed}>
            <summary>Parâmetros do registro</summary>
            <div className="grid-two">
              <div>
                <label>Série</label>
                <select value={settings.serie_digits} onChange={(e) => updateSetting('serie_digits', e.target.value)}>
                  <option value="0">Manter</option>
                  <option value="1">1 dígito</option>
                  <option value="2">2 dígitos</option>
                  <option value="3">3 dígitos</option>
                  <option value="4">4 dígitos</option>
                </select>
              </div>
              <div>
                <label>Contexto da Nota</label>
                <select value={settings.ctx_nota} onChange={(e) => updateSetting('ctx_nota', e.target.value)}>
                  <option value="entrada">Compra / Entrada</option>
                  <option value="saida">Venda / Saída</option>
                </select>
              </div>
            </div>
          </details>
        </section>
      </div>

      <section className="card nfe-card">
        <div className="nfe-card-head">
          <strong>Importar Faturas (LEGADO TXT/CSV | pipe)</strong>
        </div>
        <div className="grid-two">
          <div>
            <label>Arquivo legado</label>
            <div className="nfe-inline-actions">
              <input value={legacyFilePath} readOnly placeholder="Selecione o TXT/CSV legado" />
              <button className="btn" type="button" onClick={() => void handlePickLegacyFile()}>Selecionar</button>
            </div>
          </div>
          <div>
            <label>Ações</label>
            <div className="actions-row">
              <button className="btn primary" type="button" disabled={!legacyFilePath || processing} onClick={() => void handleImportLegacy()}>Carregar / Importar</button>
              <button className="btn danger" type="button" disabled={!rows.some((item) => item.legado)} onClick={handleClearLegacy}>Limpar importação legado</button>
            </div>
            <label className="nfe-check"><input type="checkbox" checked={legacyConferir} onChange={(e) => setLegacyConferir(e.target.checked)} /> Conferir com XML/SPED já carregados</label>
          </div>
        </div>
        <div className="grid-two">
          <div>
            <label>CNPJ da filial</label>
            <input value={legacyCnpjFilial} onChange={(e) => setLegacyCnpjFilial(onlyDigits(e.target.value))} placeholder="Somente números" />
          </div>
          <div>
            <label>Aplicação</label>
            <div className="actions-row">
              <button className="btn" type="button" onClick={() => setRows((current) => current.map((row) => row.legado ? { ...row, cnpj_filial: legacyCnpjFilial } : row))}>Aplicar às faturas legado</button>
              <button className="btn" type="button" onClick={() => void handleLegacyAutoCnpj()}>Pegar da entrada XML/SPED</button>
            </div>
          </div>
        </div>
        <details className="nfe-box" open>
          <summary>Preview da importação legado</summary>
          <div className="nfe-preview-row">
            <span className="badge">Notas: {legacyPreview.notas}</span>
            <span className="badge">Parcelas: {legacyPreview.parcelas}</span>
            <span className="badge">Inválidas: {legacyPreview.invalid_count}</span>
            <span className="badge">Avisos: {legacyPreview.warning_count}</span>
            <span className="badge">Divergências: {legacyPreview.divergence_count}</span>
          </div>
          <div className="nfe-preview-grid">
            <div>
              <strong>Linhas inválidas</strong>
              <ul className="clean-list">
                {legacyPreview.invalid_lines.length ? legacyPreview.invalid_lines.map((item) => <li key={item}>{item}</li>) : <li className="muted">Sem inconsistências.</li>}
              </ul>
            </div>
            <div>
              <strong>Avisos</strong>
              <ul className="clean-list">
                {legacyPreview.warnings.length ? legacyPreview.warnings.map((item) => <li key={item}>{item}</li>) : <li className="muted">Sem avisos.</li>}
              </ul>
            </div>
            <div>
              <strong>Divergências</strong>
              <ul className="clean-list">
                {legacyPreview.divergences.length ? legacyPreview.divergences.map((item) => <li key={item}>{item}</li>) : <li className="muted">Sem divergências.</li>}
              </ul>
            </div>
          </div>
        </details>
      </section>

      <section className="card nfe-card">
        <div className="nfe-card-head">
          <strong>Tabela de parcelas</strong>
        </div>
        <div className="inline-summary nfe-summary-row">
          <span className="badge">Registros: {gridFilter ? `${stats.count} / ${stats.total}` : stats.total}</span>
          <span className="badge">Somatório: {stats.sum}</span>
          <span className="badge">Com SPED: {stats.spedMatch}</span>
          <span className="badge">Sem SPED: {stats.spedMiss}</span>
        </div>
        <div className="nfe-inline-actions nfe-filter-row">
          <input value={gridFilter} onChange={(e) => setGridFilter(e.target.value)} placeholder="Filtrar por chave, CNPJ/CPF, número, fonte…" />
        </div>
        <div className="table-wrap nfe-table-wrap">
          <table className="grid-table nfe-grid-table">
            <thead>
              <tr>
                <th>#</th>
                <th>CHAVE</th>
                <th>DESDOB</th>
                <th>CNPJFILIAL</th>
                <th>CNPJCPF</th>
                <th>UF</th>
                <th>IE</th>
                <th>NFSERIE</th>
                <th>NFNUMERO</th>
                <th>DATAEMISSAO</th>
                <th>DATAENTRADA</th>
                <th>NUMFATURA</th>
                <th>DATAVENCIMENTO</th>
                <th>VALORBRUTOFAT</th>
                <th>FONTE</th>
              </tr>
            </thead>
            <tbody>
              {visibleRows.map((row, index) => {
                const sourceIndex = rows.indexOf(row);
                return (
                  <tr key={`${row.chave}-${row.source}-${index}`}>
                    <td>{index + 1}</td>
                    <td><input value={row.chave} onChange={(e) => handleRowChange(sourceIndex, 'chave', e.target.value)} /></td>
                    <td><input value={String(row.desdob)} onChange={(e) => handleRowChange(sourceIndex, 'desdob', e.target.value)} /></td>
                    <td><input value={row.cnpj_filial} onChange={(e) => handleRowChange(sourceIndex, 'cnpj_filial', e.target.value)} /></td>
                    <td><input value={row.cnpj_cpf} onChange={(e) => handleRowChange(sourceIndex, 'cnpj_cpf', e.target.value)} /></td>
                    <td><input value={row.uf} onChange={(e) => handleRowChange(sourceIndex, 'uf', e.target.value)} /></td>
                    <td><input value={row.ie} onChange={(e) => handleRowChange(sourceIndex, 'ie', e.target.value)} /></td>
                    <td><input value={row.nf_serie} onChange={(e) => handleRowChange(sourceIndex, 'nf_serie', e.target.value)} /></td>
                    <td><input value={row.nf_numero} onChange={(e) => handleRowChange(sourceIndex, 'nf_numero', e.target.value)} /></td>
                    <td><input value={row.data_emissao} onChange={(e) => handleRowChange(sourceIndex, 'data_emissao', e.target.value)} /></td>
                    <td><input value={row.data_entrada} onChange={(e) => handleRowChange(sourceIndex, 'data_entrada', e.target.value)} /></td>
                    <td><input value={row.num_fatura} onChange={(e) => handleRowChange(sourceIndex, 'num_fatura', e.target.value)} /></td>
                    <td><input value={row.data_vencimento} onChange={(e) => handleRowChange(sourceIndex, 'data_vencimento', e.target.value)} /></td>
                    <td><input value={row.valor_bruto_fat} onChange={(e) => handleRowChange(sourceIndex, 'valor_bruto_fat', e.target.value)} /></td>
                    <td>{row.source}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>

      {manualOpen ? (
        <dialog open className="nfe-manual-dialog">
          <div className="nfe-manual-head">
            <strong>Manual — NFe / Faturas</strong>
            <button className="btn" type="button" onClick={() => setManualOpen(false)}>Fechar</button>
          </div>
          <div className="nfe-manual-body">
            <p>Fluxo recomendado: selecionar entradas nativas do sistema → processar → revisar a grid → exportar TXT/CSV/SPED.</p>
            <ul className="clean-list">
              <li>Arquivos e pastas são escolhidos pelo diálogo nativo do Tauri.</li>
              <li>O processamento roda em Rust, inclusive ZIP, XML, SPED e legado pipe.</li>
              <li>As configurações desta tela são persistidas no diretório de dados do aplicativo.</li>
              <li>A exportação TXT/CSV grava diretamente no caminho escolhido pelo usuário.</li>
              <li>A exportação SPED gera um ou mais arquivos em uma pasta de destino.</li>
            </ul>
          </div>
        </dialog>
      ) : null}
    </div>
  );
}
