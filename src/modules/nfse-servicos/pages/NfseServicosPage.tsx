import { useEffect, useMemo, useRef, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../stores/NfseStore';
import { DocsGrid } from '../components/DocsGrid';
import { ProfileForm } from '../components/ProfileForm';
import { FieldRuleEditor } from '../components/FieldRuleEditor';
import { StatsCards } from '../components/StatsCards';
import {
  appendLog,
  exportCsv,
  exportTxt,
  loadProfileBundle,
  processNfseUploadBatch,
  saveProfileBundle,
} from '../services/tauriService';
import { downloadText } from '../../../shared/utils/download';
import { validateProfile } from '../../../shared/validators/profiles';
import type { ProfileBundle, UploadInputItem } from '../../../shared/types';
import { defaultProfileBundle } from '../../../shared/mappers/defaultProfile';

function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

async function filesToBatchItems(files: File[]): Promise<UploadInputItem[]> {
  const items: UploadInputItem[] = [];
  for (const file of files) {
    const name = file.webkitRelativePath || file.name;
    const lowerName = name.toLowerCase();
    if (lowerName.endsWith('.xml')) {
      items.push({ file_name: name, kind: 'xml', content: await file.text() });
      continue;
    }
    if (lowerName.endsWith('.zip')) {
      const bytes = new Uint8Array(await file.arrayBuffer());
      items.push({ file_name: name, kind: 'zip', content: bytesToBase64(bytes) });
    }
  }
  return items;
}

function buildDeterministicTxtFileName(outputLayout: string): string {
  const now = new Date();
  const date = `${now.getFullYear()}${`${now.getMonth() + 1}`.padStart(2, '0')}${`${now.getDate()}`.padStart(2, '0')}`;
  const time = `${`${now.getHours()}`.padStart(2, '0')}${`${now.getMinutes()}`.padStart(2, '0')}${`${now.getSeconds()}`.padStart(2, '0')}`;
  const seq = '001';

  if (outputLayout === 'ba_prestados') {
    return `LFSSERBA._${date}_${time}_${seq}.txt`;
  }

  return `${outputLayout.toUpperCase()}_${date}_${time}_${seq}.txt`;
}

export default function NfseServicosPage() {
  const { documents, profile, logs, setDocuments, setProfile, pushLog } = useNfseStore();
  const xmlInputRef = useRef<HTMLInputElement | null>(null);
  const zipInputRef = useRef<HTMLInputElement | null>(null);
  const folderInputRef = useRef<HTMLInputElement | null>(null);
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState('');
  const [profileBundle, setProfileBundle] = useState<ProfileBundle>(defaultProfileBundle);

  useEffect(() => {
    if (folderInputRef.current) {
      folderInputRef.current.setAttribute('webkitdirectory', 'true');
      folderInputRef.current.setAttribute('directory', 'true');
    }
  }, []);

  useEffect(() => {
    loadProfileBundle()
      .then((savedBundle) => {
        const nextBundle = savedBundle ?? defaultProfileBundle;
        setProfileBundle(nextBundle);
        const activeProfile =
          nextBundle.profiles.find((item) => item.profile_id === nextBundle.selected_profile_id) ||
          nextBundle.profiles[0];

        if (!activeProfile) return;

        setProfile(activeProfile);
        pushLog(
          `Perfil ativo carregado: ${activeProfile.profile_company_name || activeProfile.profile_name}.`,
        );
      })
      .catch(() => pushLog('Perfil padrão local carregado.'));
  }, [pushLog, setProfile]);

  async function processFiles(files: File[]) {
    if (!files.length) {
      pushLog('Nenhum arquivo válido foi informado.');
      return;
    }

    setBusy(true);
    try {
      const items = await filesToBatchItems(files);
      if (!items.length) {
        pushLog('Nenhum XML/ZIP encontrado na seleção.');
        return;
      }

      const result = await processNfseUploadBatch(items, profile);
      setDocuments(result.documents);
      result.warnings.forEach((warning) => pushLog(`Aviso: ${warning}`));
      result.errors.forEach((error) => pushLog(`Erro: ${error}`));
      pushLog(`Processamento concluído: ${result.documents.length} documento(s), ${result.errors.length} erro(s).`);
      await appendLog(`Lote processado com ${items.length} entrada(s).`);
    } finally {
      setBusy(false);
    }
  }

  async function onInputChange(fileList: FileList | null) {
    if (!fileList) return;
    await processFiles(Array.from(fileList));
  }

  async function handleSaveProfile() {
    const issues = validateProfile(profile);
    if (issues.length) {
      issues.forEach((issue) => pushLog(`Validação do perfil: ${issue}`));
      return;
    }

    const nextBundle: ProfileBundle = {
      selected_profile_id: profile.profile_id,
      profiles: profileBundle.profiles.map((item) =>
        item.profile_id === profile.profile_id ? profile : item,
      ),
    };

    await saveProfileBundle(nextBundle);
    setProfileBundle(nextBundle);
    pushLog(`Perfil salvo para ${profile.profile_company_name || profile.profile_name}.`);
  }

  async function handleExportTxt() {
    const txt = await exportTxt(documents, profile);
    setPreview(txt.slice(0, 12000));
    downloadText(txt, buildDeterministicTxtFileName(profile.output_layout));
    pushLog('Exportação TXT concluída.');
  }

  async function handleExportCsv() {
    const csv = await exportCsv(documents, profile);
    downloadText(csv, `${profile.cod_prosoft}_nfse_conferencia.csv`, 'text/csv;charset=utf-8');
    pushLog('Exportação CSV concluída.');
  }

  const warningsCount = useMemo(
    () => documents.reduce((acc, item) => acc + item.warnings.length, 0),
    [documents],
  );

  return (
    <div className="stack-lg">
      <PageHeader
        title="NFS-e → Prosoft"
        subtitle="Importe XML, ZIP ou pasta usando o perfil de empresa atualmente selecionado."
        actions={(
          <div className="actions-row">
            <button className="btn primary" onClick={() => xmlInputRef.current?.click()} disabled={busy}>Selecionar XML(s)</button>
            <button className="btn" onClick={() => zipInputRef.current?.click()} disabled={busy}>Selecionar ZIP</button>
            <button className="btn" onClick={() => folderInputRef.current?.click()} disabled={busy}>Selecionar Pasta</button>
            <button className="btn" onClick={handleSaveProfile} disabled={busy}>Salvar perfil ativo</button>
            <button className="btn success" onClick={handleExportTxt} disabled={busy || documents.length === 0}>Exportar TXT</button>
            <button className="btn" onClick={handleExportCsv} disabled={busy || documents.length === 0}>Exportar CSV</button>
          </div>
        )}
      />

      <input ref={xmlInputRef} className="hidden" type="file" accept=".xml,text/xml,application/xml" multiple onChange={(e) => onInputChange(e.target.files)} />
      <input ref={zipInputRef} className="hidden" type="file" accept=".zip,application/zip" multiple onChange={(e) => onInputChange(e.target.files)} />
      <input ref={folderInputRef} className="hidden" type="file" multiple onChange={(e) => onInputChange(e.target.files)} />

      <div className={`dropzone ${busy ? 'disabled' : ''}`} onDragOver={(event) => event.preventDefault()} onDrop={async (event) => {
        event.preventDefault();
        if (busy) return;
        await processFiles(Array.from(event.dataTransfer.files || []));
      }}>
        <strong>Arraste XML, ZIP ou uma pasta inteira para processamento em lote</strong>
        <span className="muted">O sistema valida item a item e mantém o lote quando houver erro pontual.</span>
      </div>

      <StatsCards documents={documents} />

      <div className="card compact-card">
        <h3>Perfil ativo</h3>
        <p className="muted">
          Perfil selecionado: <b>{profile.profile_company_name || profile.profile_name}</b>. Os perfis são independentes do licenciamento da aplicação.
        </p>
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
          <p className="muted">Avisos identificados no lote: {warningsCount}</p>
          <pre className="console-box">{logs.join('\n') || '(sem logs)'}</pre>
        </div>
      </div>
    </div>
  );
}
