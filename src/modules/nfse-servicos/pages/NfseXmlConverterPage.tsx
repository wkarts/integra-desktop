import { useMemo, useRef, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../stores/NfseStore';
import {
  convertNfseMixedBatchToStandard,
  convertNfseXmlToStandard,
  dialogPickNfseConverterDirectory,
} from '../services/tauriService';
import { downloadText } from '../../../shared/utils/download';
import type {
  StandardizeXmlOptions,
  StandardizedXmlBatchResult,
  StandardizedXmlResult,
  StandardizedXmlTarget,
  UploadInputItem,
} from '../../../shared/types';

const defaultOptions: StandardizeXmlOptions = {
  target: 'abrasf_v2',
  remove_iss_aliquota: false,
  remove_iss_value: false,
  keep_only_iss_retido: false,
  remove_incompatible_tags: true,
  apply_profile_rules: true,
  remove_codigo_verificacao: false,
  remove_tomador_endereco: false,
  remove_prestador_im: false,
  remove_tomador_im: false,
  remove_cnae: false,
  remove_discriminacao: false,
  remove_info_adicional: false,
};

function buildFileName(target: StandardizedXmlTarget) {
  const now = new Date();
  const stamp = `${now.getFullYear()}${`${now.getMonth() + 1}`.padStart(2, '0')}${`${now.getDate()}`.padStart(2, '0')}_${`${now.getHours()}`.padStart(2, '0')}${`${now.getMinutes()}`.padStart(2, '0')}${`${now.getSeconds()}`.padStart(2, '0')}`;
  return `NFSE_${target}_${stamp}.xml`;
}

function downloadBase64(base64: string, fileName: string, mime = 'application/octet-stream') {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  const blob = new Blob([bytes], { type: mime });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = fileName;
  anchor.click();
  setTimeout(() => URL.revokeObjectURL(url), 1500);
}

async function fileToUploadItem(file: File): Promise<UploadInputItem> {
  const lower = file.name.toLowerCase();
  if (lower.endsWith('.zip')) {
    const bytes = await file.arrayBuffer();
    let binary = '';
    const array = new Uint8Array(bytes);
    array.forEach((item) => {
      binary += String.fromCharCode(item);
    });
    return {
      file_name: file.name,
      kind: 'zip',
      content: btoa(binary),
    };
  }

  return {
    file_name: file.name,
    kind: 'xml',
    content: await file.text(),
  };
}

export default function NfseXmlConverterPage() {
  const { profile, pushLog } = useNfseStore();
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const batchInputRef = useRef<HTMLInputElement | null>(null);
  const [busy, setBusy] = useState(false);
  const [fileName, setFileName] = useState('');
  const [sourceXml, setSourceXml] = useState('');
  const [options, setOptions] = useState<StandardizeXmlOptions>(defaultOptions);
  const [result, setResult] = useState<StandardizedXmlResult | null>(null);
  const [batchResult, setBatchResult] = useState<StandardizedXmlBatchResult | null>(null);
  const [selectedBatchItems, setSelectedBatchItems] = useState<string[]>([]);
  const [selectedDirectories, setSelectedDirectories] = useState<string[]>([]);

  async function handlePickFile(file: File | null) {
    if (!file) return;
    const text = await file.text();
    setFileName(file.name);
    setSourceXml(text);
    pushLog(`XML carregado no conversor: ${file.name}.`);
  }

  async function handleConvert() {
    if (!sourceXml.trim()) {
      pushLog('Informe um XML de NFS-e para converter.');
      return;
    }

    setBusy(true);
    try {
      const converted = await convertNfseXmlToStandard(
        sourceXml,
        fileName || 'nfse.xml',
        profile,
        options,
      );
      setResult(converted);
      converted.warnings.forEach((warning) => pushLog(`Conversor XML: ${warning}`));
      pushLog(`XML convertido com sucesso. Layout detectado: ${converted.detected_layout}.`);
    } finally {
      setBusy(false);
    }
  }

  async function handlePickBatchFiles(files: FileList | null) {
    if (!files?.length) return;
    setSelectedBatchItems(Array.from(files).map((file) => file.name));
    pushLog(`${files.length} arquivo(s) selecionado(s) para portabilidade em lote.`);
  }

  async function handlePickDirectory() {
    const directory = await dialogPickNfseConverterDirectory();
    if (!directory) return;
    setSelectedDirectories((current) => Array.from(new Set([...current, directory])));
    pushLog(`Diretório selecionado para portabilidade em lote: ${directory}.`);
  }

  async function handleBatchConvert() {
    const fileList = batchInputRef.current?.files;
    if ((!fileList || fileList.length === 0) && selectedDirectories.length === 0) {
      pushLog('Selecione ao menos um XML/ZIP ou um diretório para portabilidade em lote.');
      return;
    }

    setBusy(true);
    try {
      const uploadItems = fileList?.length ? await Promise.all(Array.from(fileList).map(fileToUploadItem)) : [];
      const combined = await convertNfseMixedBatchToStandard(uploadItems, selectedDirectories, profile, options);

      setBatchResult(combined);
      combined.warnings.forEach((warning) => pushLog(`Portabilidade em lote: ${warning}`));
      combined.errors.forEach((error) => pushLog(`Portabilidade em lote: ${error}`));
      pushLog(`Portabilidade em lote concluída: ${combined.entries.length} XML(s) convertido(s).`);
    } finally {
      setBusy(false);
    }
  }

  const metadata = useMemo(() => {
    if (!result) return null;
    return [
      ['Layout detectado', result.detected_layout],
      ['Provider', result.provider],
      ['Número', result.document.numero],
      ['Município', result.document.municipio_nome || result.document.municipio_codigo || '—'],
      ['Tomador', result.document.tomador.nome || '—'],
    ];
  }, [result]);

  return (
    <div className="stack-lg">
      <PageHeader
        title="Conversor XML NFS-e"
        subtitle="Faça portabilidade unitária ou em lote, inclusive para o mesmo layout padronizado, aplicando as regras do perfil da empresa para remover, manter ou fixar valores em massa."
        actions={(
          <div className="actions-row" style={{ flexWrap: 'wrap' }}>
            <button className="btn" onClick={() => fileInputRef.current?.click()} disabled={busy}>Selecionar XML unitário</button>
            <button className="btn primary" onClick={handleConvert} disabled={busy || !sourceXml.trim()}>Converter XML</button>
            <button
              className="btn success"
              onClick={() => result && downloadText(result.standardized_xml, buildFileName(options.target), 'application/xml;charset=utf-8')}
              disabled={!result}
            >
              Baixar XML padronizado
            </button>
            <button className="btn" onClick={() => batchInputRef.current?.click()} disabled={busy}>Selecionar XML/ZIP em lote</button>
            <button className="btn" onClick={handlePickDirectory} disabled={busy}>Selecionar diretório</button>
            <button className="btn primary" onClick={handleBatchConvert} disabled={busy}>Portar em lote</button>
            <button
              className="btn success"
              onClick={() => batchResult && downloadBase64(batchResult.zip_base64, batchResult.zip_file_name, 'application/zip')}
              disabled={!batchResult?.entries.length}
            >
              Baixar lote ZIP
            </button>
          </div>
        )}
      />

      <input
        ref={fileInputRef}
        className="hidden"
        type="file"
        accept=".xml,text/xml,application/xml"
        onChange={(e) => handlePickFile(e.target.files?.[0] ?? null)}
      />

      <input
        ref={batchInputRef}
        className="hidden"
        type="file"
        accept=".xml,.zip,text/xml,application/xml,application/zip"
        multiple
        onChange={(e) => handlePickBatchFiles(e.target.files)}
      />

      <div className="grid-two">
        <div className="card">
          <h3>Opções de portabilidade</h3>
          <div className="form-grid two-columns">
            <label className="field-group">
              <span>Layout de destino</span>
              <select value={options.target} onChange={(e) => setOptions((current) => ({ ...current, target: e.target.value as StandardizedXmlTarget }))}>
                <option value="abrasf_v2">ABRASF v2</option>
                <option value="abrasf_v1">ABRASF v1</option>
                <option value="salvador_like">Compatível Salvador</option>
                <option value="same_layout">Mesmo layout detectado</option>
              </select>
            </label>
          </div>

          <div className="checkbox-grid" style={{ marginTop: 12 }}>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.apply_profile_rules)} onChange={(e) => setOptions((current) => ({ ...current, apply_profile_rules: e.target.checked }))} /> Aplicar regras do perfil da empresa</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_iss_aliquota)} onChange={(e) => setOptions((current) => ({ ...current, remove_iss_aliquota: e.target.checked }))} /> Remover alíquota ISS</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_iss_value)} onChange={(e) => setOptions((current) => ({ ...current, remove_iss_value: e.target.checked }))} /> Remover valor ISS</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.keep_only_iss_retido)} onChange={(e) => setOptions((current) => ({ ...current, keep_only_iss_retido: e.target.checked }))} /> Manter apenas ISS retido</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_incompatible_tags)} onChange={(e) => setOptions((current) => ({ ...current, remove_incompatible_tags: e.target.checked }))} /> Remover tags incompatíveis</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_codigo_verificacao)} onChange={(e) => setOptions((current) => ({ ...current, remove_codigo_verificacao: e.target.checked }))} /> Remover código de verificação</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_tomador_endereco)} onChange={(e) => setOptions((current) => ({ ...current, remove_tomador_endereco: e.target.checked }))} /> Remover endereço do tomador</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_prestador_im)} onChange={(e) => setOptions((current) => ({ ...current, remove_prestador_im: e.target.checked }))} /> Remover IM do prestador</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_tomador_im)} onChange={(e) => setOptions((current) => ({ ...current, remove_tomador_im: e.target.checked }))} /> Remover IM do tomador</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_cnae)} onChange={(e) => setOptions((current) => ({ ...current, remove_cnae: e.target.checked }))} /> Remover CNAE</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_discriminacao)} onChange={(e) => setOptions((current) => ({ ...current, remove_discriminacao: e.target.checked }))} /> Remover discriminação</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_info_adicional)} onChange={(e) => setOptions((current) => ({ ...current, remove_info_adicional: e.target.checked }))} /> Remover informações adicionais</label>
          </div>
        </div>

        <div className="card">
          <h3>Resumo da conversão</h3>
          {!metadata ? (
            <p className="muted">Nenhuma conversão executada ainda.</p>
          ) : (
            <div className="form-grid two-columns">
              {metadata.map(([label, value]) => (
                <label key={label} className="field-group">
                  <span>{label}</span>
                  <input value={value} readOnly />
                </label>
              ))}
            </div>
          )}
          <div style={{ marginTop: 16 }}>
            <h4>Itens prontos para lote</h4>
            <p className="muted">Arquivos: {selectedBatchItems.length} | Diretórios: {selectedDirectories.length}</p>
            {selectedBatchItems.length > 0 && <p className="muted">{selectedBatchItems.slice(0, 5).join(', ')}{selectedBatchItems.length > 5 ? ' ...' : ''}</p>}
            {selectedDirectories.length > 0 && <p className="muted">{selectedDirectories.join(' | ')}</p>}
            {batchResult && (
              <p className="muted">Último lote: {batchResult.entries.length} convertido(s), {batchResult.errors.length} erro(s), {batchResult.warnings.length} aviso(s).</p>
            )}
          </div>
        </div>
      </div>

      <div className="grid-two">
        <div className="card">
          <h3>XML de origem</h3>
          <textarea className="textarea-mono" rows={22} value={sourceXml} onChange={(e) => setSourceXml(e.target.value)} placeholder="Cole aqui o XML original de NFS-e ou selecione um arquivo." />
        </div>
        <div className="card">
          <h3>XML padronizado</h3>
          <textarea className="textarea-mono" rows={22} value={result?.standardized_xml || ''} readOnly placeholder="O XML convertido será exibido aqui." />
        </div>
      </div>

      <div className="card">
        <h3>Resultado da portabilidade em lote</h3>
        {!batchResult ? (
          <p className="muted">Nenhum lote processado ainda.</p>
        ) : (
          <div className="stack-md">
            <div className="form-grid three-columns">
              <label className="field-group"><span>Convertidos</span><input value={String(batchResult.entries.length)} readOnly /></label>
              <label className="field-group"><span>Avisos</span><input value={String(batchResult.warnings.length)} readOnly /></label>
              <label className="field-group"><span>Erros</span><input value={String(batchResult.errors.length)} readOnly /></label>
            </div>
            <div style={{ maxHeight: 280, overflow: 'auto' }}>
              <table className="table-like">
                <thead>
                  <tr>
                    <th>Origem</th>
                    <th>Layout detectado</th>
                    <th>Provider</th>
                    <th>Saída</th>
                  </tr>
                </thead>
                <tbody>
                  {batchResult.entries.map((entry) => (
                    <tr key={`${entry.source_name}-${entry.output_file_name}`}>
                      <td>{entry.source_name}</td>
                      <td>{entry.detected_layout}</td>
                      <td>{entry.provider}</td>
                      <td>{entry.output_file_name}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            {batchResult.errors.length > 0 && (
              <div>
                <h4>Erros</h4>
                <textarea className="textarea-mono" rows={8} readOnly value={batchResult.errors.join('\n')} />
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
