import { useEffect, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import type { AppMeta, LicenseRuntimeStatus, ProfileBundle } from '../../../shared/types';
import { checkLicenseStatus, getAppMeta, loadLicenseSettings, loadProfileBundle } from '../../nfse-servicos/services/tauriService';

const defaultMeta: AppMeta = {
  product_name: 'Integra Desktop',
  version: '1.0.3',
  build_hash: 'dev-local',
  app_id: 'integra-desktop',
};

export default function DashboardPage() {
  const [bundle, setBundle] = useState<ProfileBundle | null>(null);
  const [license, setLicense] = useState<LicenseRuntimeStatus | null>(null);
  const [meta, setMeta] = useState<AppMeta>(defaultMeta);
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    loadProfileBundle().then(setBundle).catch(() => setBundle(null));
    getAppMeta().then(setMeta).catch(() => setMeta(defaultMeta));
    loadLicenseSettings()
      .then(async (settings) => {
        if (settings?.company_document && settings?.service_url) {
          setLicense(await checkLicenseStatus(settings));
        }
      })
      .catch(() => setLicense(null));
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const profileCount = bundle?.profiles.length ? 1 : 0;
  const selectedProfile = bundle?.profiles.find((item) => item.profile_id === bundle?.selected_profile_id);
  const hour = now.getHours() % 12;
  const minute = now.getMinutes();
  const second = now.getSeconds();
  const hourAngle = hour * 30 + minute * 0.5;
  const minuteAngle = minute * 6 + second * 0.1;
  const secondAngle = second * 6;
  const timeLabel = now.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' });
  const dateLabel = now.toLocaleDateString('pt-BR', { weekday: 'long', day: '2-digit', month: 'short' });

  return (
    <div className="stack-lg">
      <PageHeader
        title="Dashboard operacional"
        subtitle="Visão geral da operação."
        actions={(
          <section className="dashboard-system-panel dashboard-system-panel-mobile" aria-label="Relógio e versão do sistema">
            <div className="dashboard-clock" role="img" aria-label={`Horário atual: ${timeLabel}`}>
              <span className="clock-hand clock-hand-hour" style={{ transform: `translateX(-50%) rotate(${hourAngle}deg)` }} />
              <span className="clock-hand clock-hand-minute" style={{ transform: `translateX(-50%) rotate(${minuteAngle}deg)` }} />
              <span className="clock-hand clock-hand-second" style={{ transform: `translateX(-50%) rotate(${secondAngle}deg)` }} />
              <span className="clock-center" />
            </div>
            <div className="dashboard-system-meta">
              <strong className="clock-time">{timeLabel}</strong>
              <span className="clock-date">{dateLabel}</span>
              <div className="dashboard-version">
                <span>Versão {meta.version}</span>
                <span>ASHA {meta.build_hash.slice(0, 12)}</span>
              </div>
            </div>
          </section>
        )}
      />

      <div className="kpi-grid kpi-grid-4">
        <div className="card kpi-card"><span>Razão social</span><strong>{license?.company_name || selectedProfile?.user_company_name || 'Não configurada'}</strong><p className="muted">Cadastro licenciado da aplicação.</p></div>
        <div className="card kpi-card"><span>Configuração de empresa</span><strong>{profileCount}</strong><p className="muted">A aplicação utiliza um único layout por empresa.</p></div>
        <div className="card kpi-card"><span>Empresa ativa</span><strong>{selectedProfile?.profile_company_name || selectedProfile?.profile_name || 'Não selecionado'}</strong><p className="muted">Configuração usada na próxima conversão.</p></div>
        <div className="card kpi-card"><span>Status da licença</span><strong>{license ? (license.allowed ? 'Liberada' : 'Bloqueada') : 'Pendente'}</strong><p className="muted">{license?.message || 'Valide em Configurações.'}</p></div>
      </div>

      <div className="grid-two dashboard-grid">
        <div className="card">
          <h3>Fluxo principal</h3>
          <ol className="clean-list">
            <li>Cadastre empresa e valide a licença.</li>
            <li>Cadastre a empresa, município e layout municipal de NFS-e.</li>
            <li>Importe XML, ZIP ou pasta em <b>NFS-e → Prosoft</b>.</li>
            <li>Revise o lote e exporte TXT/CSV com a configuração ativa.</li>
          </ol>
        </div>

        <div className="card">
          <h3>Licenciamento por estação</h3>
          <p className="muted">Controle centralizado por webservice.</p>
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
