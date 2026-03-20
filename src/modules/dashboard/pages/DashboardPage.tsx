import { useEffect, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import type { LicenseRuntimeStatus, ProfileBundle } from '../../../shared/types';
import { checkLicenseStatus, loadLicenseSettings, loadProfileBundle } from '../../nfse-servicos/services/tauriService';

export default function DashboardPage() {
  const [bundle, setBundle] = useState<ProfileBundle | null>(null);
  const [license, setLicense] = useState<LicenseRuntimeStatus | null>(null);

  useEffect(() => {
    loadProfileBundle().then(setBundle).catch(() => setBundle(null));
    loadLicenseSettings()
      .then(async (settings) => {
        if (settings?.company_document && settings?.service_url) {
          setLicense(await checkLicenseStatus(settings));
        }
      })
      .catch(() => setLicense(null));
  }, []);

  const profileCount = bundle?.profiles.length ?? 0;
  const selectedProfile = bundle?.profiles.find((item) => item.profile_id === bundle?.selected_profile_id);

  return (
    <div className="stack-lg">
      <PageHeader title="Painel operacional" subtitle="Entrada de documentos, perfis por empresa e acompanhamento do licenciamento da estação." />

      <div className="kpi-grid kpi-grid-4">
        <div className="card kpi-card"><span>Razão social</span><strong>{license?.company_name || selectedProfile?.user_company_name || 'Não configurada'}</strong><p className="muted">Cadastro licenciado da aplicação.</p></div>
        <div className="card kpi-card"><span>Perfis de escrituração</span><strong>{profileCount}</strong><p className="muted">Empresas/perfis disponíveis para exportação.</p></div>
        <div className="card kpi-card"><span>Perfil ativo</span><strong>{selectedProfile?.profile_company_name || selectedProfile?.profile_name || 'Não selecionado'}</strong><p className="muted">Perfil usado na próxima conversão.</p></div>
        <div className="card kpi-card"><span>Status da licença</span><strong>{license ? (license.allowed ? 'Liberada' : 'Bloqueada') : 'Pendente'}</strong><p className="muted">{license?.message || 'Configure o webservice em Configurações.'}</p></div>
      </div>

      <div className="grid-two dashboard-grid">
        <div className="card">
          <h3>Fluxo principal</h3>
          <ol className="clean-list">
            <li>Configure os dados da empresa e o licenciamento.</li>
            <li>Cadastre um ou mais perfis com o nome da empresa para a qual fará a escrituração.</li>
            <li>Importe XML, ZIP ou pasta em <b>NFS-e → Prosoft</b>.</li>
            <li>Revise o lote, escolha o perfil correto e exporte TXT/CSV.</li>
          </ol>
        </div>

        <div className="card">
          <h3>Licenciamento por estação</h3>
          <p className="muted">A estação utiliza chave de máquina e validação centralizada pelo webservice. As máquinas não precisam se enxergar na rede local.</p>
          <div className="inline-summary">
            <span>Total liberado: <b>{license?.seats_total ?? 0}</b></span>
            <span>Em uso: <b>{license?.seats_used ?? 0}</b></span>
            <span>Chave atual: <b>{license?.machine_key?.slice(0, 12) || '—'}</b></span>
            <span>Validade: <b>{license?.expiry || 'N/D'}</b></span>
          </div>
        </div>
      </div>
    </div>
  );
}
