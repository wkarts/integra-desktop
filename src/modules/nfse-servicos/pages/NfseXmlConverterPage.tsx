import { useMemo, useRef, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../stores/NfseStore';
import { convertNfseXmlToStandard } from '../services/tauriService';
import { downloadText } from '../../../shared/utils/download';
import type { StandardizeXmlOptions, StandardizedXmlResult, StandardizedXmlTarget } from '../../../shared/types';

const defaultOptions: StandardizeXmlOptions = {
  target: 'abrasf_v2',
  remove_iss_aliquota: false,
  remove_iss_value: false,
  keep_only_iss_retido: false,
  remove_incompatible_tags: true,
};

function buildFileName(target: StandardizedXmlTarget) {
  const now = new Date();
  const stamp = `${now.getFullYear()}${`${now.getMonth() + 1}`.padStart(2, '0')}${`${now.getDate()}`.padStart(2, '0')}_${`${now.getHours()}`.padStart(2, '0')}${`${now.getMinutes()}`.padStart(2, '0')}${`${now.getSeconds()}`.padStart(2, '0')}`;
  return `NFSE_${target}_${stamp}.xml`;
}

export default function NfseXmlConverterPage() {
  const { profile, pushLog } = useNfseStore();
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [busy, setBusy] = useState(false);
  const [fileName, setFileName] = useState('');
  const [sourceXml, setSourceXml] = useState('');
  const [options, setOptions] = useState<StandardizeXmlOptions>(defaultOptions);
  const [result, setResult] = useState<StandardizedXmlResult | null>(null);

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
        subtitle="Converta XML municipal para um XML padronizado ABRASF/compatível sem alterar a origem real da nota."
        actions={(
          <div className="actions-row">
            <button className="btn" onClick={() => fileInputRef.current?.click()} disabled={busy}>Selecionar XML</button>
            <button className="btn primary" onClick={handleConvert} disabled={busy || !sourceXml.trim()}>Converter XML</button>
            <button
              className="btn success"
              onClick={() => result && downloadText(result.standardized_xml, buildFileName(options.target), 'application/xml;charset=utf-8')}
              disabled={!result}
            >
              Baixar XML padronizado
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

      <div className="grid-two">
        <div className="card">
          <h3>Opções de conversão</h3>
          <div className="form-grid two-columns">
            <label className="field-group">
              <span>Layout de destino</span>
              <select
                value={options.target}
                onChange={(e) => setOptions((current) => ({ ...current, target: e.target.value as StandardizedXmlTarget }))}
              >
                <option value="abrasf_v2">ABRASF v2</option>
                <option value="abrasf_v1">ABRASF v1</option>
                <option value="salvador_like">Compatível Salvador</option>
              </select>
            </label>
          </div>

          <div className="checkbox-grid" style={{ marginTop: 12 }}>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_iss_aliquota)} onChange={(e) => setOptions((current) => ({ ...current, remove_iss_aliquota: e.target.checked }))} /> Remover alíquota ISS</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_iss_value)} onChange={(e) => setOptions((current) => ({ ...current, remove_iss_value: e.target.checked }))} /> Remover valor ISS</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.keep_only_iss_retido)} onChange={(e) => setOptions((current) => ({ ...current, keep_only_iss_retido: e.target.checked }))} /> Manter apenas ISS retido</label>
            <label className="checkbox-inline"><input type="checkbox" checked={Boolean(options.remove_incompatible_tags)} onChange={(e) => setOptions((current) => ({ ...current, remove_incompatible_tags: e.target.checked }))} /> Remover tags incompatíveis</label>
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
        </div>
      </div>

      <div className="grid-two">
        <div className="card">
          <h3>XML de origem</h3>
          <textarea
            className="textarea-mono"
            rows={22}
            value={sourceXml}
            onChange={(e) => setSourceXml(e.target.value)}
            placeholder="Cole aqui o XML original de NFS-e ou selecione um arquivo."
          />
        </div>
        <div className="card">
          <h3>XML padronizado</h3>
          <textarea
            className="textarea-mono"
            rows={22}
            value={result?.standardized_xml || ''}
            readOnly
            placeholder="O XML convertido será exibido aqui."
          />
        </div>
      </div>
    </div>
  );
}
