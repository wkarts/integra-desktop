import { useEffect, useMemo, useRef, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../stores/NfseStore';
import { DocsGrid } from '../components/DocsGrid';
import { ProfileForm } from '../components/ProfileForm';
import { FieldRuleEditor } from '../components/FieldRuleEditor';
import { StatsCards } from '../components/StatsCards';
import { appendLog, exportCsv, exportTxt, loadProfile, processNfseBatch, saveProfile } from '../services/tauriService';
import { downloadText } from '../../../shared/utils/download';
import { validateProfile } from '../../../shared/validators/profiles';
import type { ProcessBatchInputItem } from '../../../shared/types';

export default function NfseServicosPage() {
  const { documents, profile, logs, setDocuments, setProfile, pushLog } = useNfseStore();
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState('');

  useEffect(() => {
    loadProfile().then((saved) => {
      if (saved) {
        setProfile(saved);
        pushLog('Perfil salvo carregado do backend Tauri.');
      }
    }).catch(() => {
      pushLog('Perfil padrão local carregado.');
    });
  }, [pushLog, setProfile]);

  async function handleOpenFiles(fileList: FileList | null) {
    if (!fileList?.length) {
      return;
    }

    setBusy(true);
    try {
      const items: ProcessBatchInputItem[] = [];
      for (const file of Array.from(fileList)) {
        if (!file.name.toLowerCase().endsWith('.xml')) continue;
        items.push({ file_name: file.name, xml: await file.text() });
      }

      const result = await processNfseBatch(items);
      setDocuments(result.documents);
      result.warnings.forEach((warning) => pushLog(`Aviso: ${warning}`));
      result.errors.forEach((error) => pushLog(`Erro: ${error}`));
      pushLog(`${result.documents.length} XML(s) processado(s) pelo core Rust.`);
      await appendLog(`Processamento concluído com ${result.documents.length} documento(s).`);
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveProfile() {
    const issues = validateProfile(profile);
    if (issues.length) {
      issues.forEach((issue) => pushLog(`Validação do perfil: ${issue}`));
      return;
    }
    await saveProfile(profile);
    pushLog('Perfil de conversão salvo no armazenamento do Tauri.');
  }

  async function handleExportTxt() {
    const txt = await exportTxt(documents, profile);
    setPreview(txt.slice(0, 12000));
    downloadText(txt, `${profile.cod_prosoft}_nfse_${profile.output_layout}.txt`);
    pushLog('TXT Prosoft exportado pelo backend Rust.');
  }

  async function handleExportCsv() {
    const csv = await exportCsv(documents, profile);
    downloadText(csv, `${profile.cod_prosoft}_nfse_conferencia.csv`, 'text/csv;charset=utf-8');
    pushLog('CSV exportado pelo backend Rust.');
  }

  const warningsCount = useMemo(() => documents.reduce((acc, item) => acc + item.warnings.length, 0), [documents]);

  return (
    <div className="stack-lg">
      <PageHeader
        title="NFS-e → Prosoft"
        subtitle="Motor principal em Rust/Tauri com fallback legado HTML separado. Suporte inicial implementado para WebISS/SAJ e Ubaíra/BA."
        actions={(
          <div className="actions-row">
            <button className="btn primary" onClick={() => fileInputRef.current?.click()} disabled={busy}>Selecionar XMLs</button>
            <button className="btn" onClick={handleSaveProfile} disabled={busy}>Salvar perfil</button>
            <button className="btn success" onClick={handleExportTxt} disabled={busy || documents.length === 0}>Exportar TXT</button>
            <button className="btn" onClick={handleExportCsv} disabled={busy || documents.length === 0}>Exportar CSV</button>
          </div>
        )}
      />

      <input ref={fileInputRef} className="hidden" type="file" accept=".xml,text/xml,application/xml" multiple onChange={(e) => handleOpenFiles(e.target.files)} />

      <StatsCards documents={documents} />

      <div className="alert-strip">
        <strong>Recurso solicitado implementado:</strong> regras por campo para zerar, anular em branco, ignorar ou fixar valor de ISS, base de cálculo,
        retenções, observação e código de serviço durante a conversão.
      </div>

      <ProfileForm value={profile} onChange={setProfile} />
      <FieldRuleEditor value={profile.field_rules} onChange={(rules) => setProfile({ ...profile, field_rules: rules })} />
      <DocsGrid documents={documents} onDocumentsChange={setDocuments} />

      <div className="grid-two">
        <div className="card">
          <h3>Prévia do TXT</h3>
          <pre className="console-box">{preview || '(sem saída ainda)'}</pre>
        </div>
        <div className="card">
          <h3>Log de operação</h3>
          <p className="muted">Warnings detectados: {warningsCount}</p>
          <pre className="console-box">{logs.join('\n') || '(sem logs)'}</pre>
        </div>
      </div>
    </div>
  );
}
