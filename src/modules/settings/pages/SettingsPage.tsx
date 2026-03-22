import { useEffect, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../../nfse-servicos/stores/NfseStore';
import { ProfileForm } from '../../nfse-servicos/components/ProfileForm';
import { FieldRuleEditor } from '../../nfse-servicos/components/FieldRuleEditor';
import {
  checkLicenseStatus,
  getAppMeta,
  getMachineFingerprint,
  loadLicenseSettings,
  saveLicenseSettings,
  loadProfile,
  saveProfile,
} from '../../nfse-servicos/services/tauriService';
import type { AppMeta, LicenseRuntimeStatus, LicenseSettings } from '../../../shared/types';
import { validateProfile } from '../../../shared/validators/profiles';

const defaultLicenseSettings: LicenseSettings = {
  service_url: 'https://api.rest.wwsoftwares.com.br/api/v1',
  company_name: '',
  company_document: '',
  company_email: '',
  station_name: '',
  machine_key: '',
  auto_register_machine: true,
  app_instance: 'integra-desktop',
};

const defaultMeta: AppMeta = {
  product_name: 'Integra Desktop',
  version: '1.4.1',
  build_hash: 'dev-local',
  app_id: 'integra-desktop',
};

export default function SettingsPage() {
  const { profile, setProfile, pushLog } = useNfseStore();
  const [licenseSettings, setLicenseSettings] = useState<LicenseSettings>(defaultLicenseSettings);
  const [licenseStatus, setLicenseStatus] = useState<LicenseRuntimeStatus | null>(null);
  const [meta, setMeta] = useState<AppMeta>(defaultMeta);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    Promise.all([loadLicenseSettings(), getMachineFingerprint(), loadProfile(), getAppMeta()])
      .then(([savedLicense, fingerprint, savedProfile, appMeta]) => {
        setMeta(appMeta);
        setLicenseSettings(savedLicense ?? { ...defaultLicenseSettings, machine_key: fingerprint });
        if (savedProfile) setProfile(savedProfile);
      })
      .catch(() => pushLog('Falha ao carregar configurações locais.'));
  }, [pushLog, setProfile]);

  async function saveAllSettings() {
    const issues = validateProfile(profile);
    if (issues.length) {
      issues.forEach((issue) => pushLog(`Validação do perfil: ${issue}`));
      return;
    }

    setBusy(true);
    try {
      const persistedLicense = await saveLicenseSettings({
        ...licenseSettings,
        company_name: licenseSettings.company_name || profile.user_company_name,
        company_document: licenseSettings.company_document || profile.user_company_document,
      });
      setLicenseSettings(persistedLicense);

      const nextProfile = {
        ...profile,
        user_company_name: persistedLicense.company_name,
        user_company_document: persistedLicense.company_document,
      };
      setProfile(nextProfile);

      await saveProfile(nextProfile);
      pushLog('Configurações gerais e empresa salvas com sucesso.');
    } finally {
      setBusy(false);
    }
  }

  async function handleCheckLicense() {
    setBusy(true);
    try {
      const result = await checkLicenseStatus(licenseSettings);
      setLicenseStatus(result);
      if (!result.allowed) {
        pushLog(`Licenciamento bloqueado: ${result.message} (${result.block_reason ?? 'sem motivo informado'})`);
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="stack-lg">
      <PageHeader
        title="Configurações"
        subtitle="Empresa, licença e layout municipal de NFS-e."
        actions={(
          <div className="actions-row">
            <button className="btn" onClick={handleCheckLicense} disabled={busy}>Validar licença</button>
            <button className="btn primary" onClick={saveAllSettings} disabled={busy}>Salvar tudo</button>
          </div>
        )}
      />

      <div className="card">
        <div className="section-title-row">
          <div>
            <h3>Licenciamento da empresa</h3>
            <p className="muted">Dados da empresa e status da licença.</p>
          </div>
          <div className="meta-badges">
            <span className="badge">Versão {meta.version}</span>
            <span className="badge">ASHA {meta.build_hash.slice(0, 12)}</span>
          </div>
        </div>

        <div className="form-grid cols-4">
          <div>
            <label>Razão social</label>
            <input value={licenseSettings.company_name} onChange={(e) => setLicenseSettings({ ...licenseSettings, company_name: e.target.value })} />
          </div>
          <div>
            <label>CNPJ/CPF</label>
            <input value={licenseSettings.company_document} onChange={(e) => setLicenseSettings({ ...licenseSettings, company_document: e.target.value })} />
          </div>
          <div>
            <label>E-mail</label>
            <input value={licenseSettings.company_email} onChange={(e) => setLicenseSettings({ ...licenseSettings, company_email: e.target.value })} />
          </div>
          <div>
            <label>Nome da estação</label>
            <input value={licenseSettings.station_name} onChange={(e) => setLicenseSettings({ ...licenseSettings, station_name: e.target.value })} placeholder="Financeiro-01" />
          </div>
          <div>
            <label>Chave da máquina</label>
            <input value={licenseSettings.machine_key} readOnly />
          </div>
        </div>

        <div className="status-panel">
          <div className="status-item"><span>Status</span><strong>{licenseStatus ? (licenseStatus.allowed ? 'Liberada' : 'Bloqueada') : 'Não verificada'}</strong></div>
          <div className="status-item"><span>Máquinas liberadas</span><strong>{licenseStatus?.seats_total ?? 0}</strong></div>
          <div className="status-item"><span>Máquinas em uso</span><strong>{licenseStatus?.seats_used ?? 0}</strong></div>
          <div className="status-item"><span>Retorno</span><strong>{licenseStatus?.message || 'Ainda não consultado'}</strong></div>
        </div>

        {licenseStatus && (
          <div className="form-grid cols-4" style={{ marginTop: 16 }}>
            <div>
              <label>Empresa remota (IDCLIENTE)</label>
              <input readOnly value={licenseStatus.licensed_company?.idcliente ?? 0} />
            </div>
            <div>
              <label>Máquina remota (IDMAQUINA)</label>
              <input readOnly value={licenseStatus.licensed_device?.idmaquina ?? 0} />
            </div>
            <div>
              <label>Validade</label>
              <input readOnly value={licenseStatus.expiry || 'Não informada'} />
            </div>
            <div>
              <label>Motivo técnico</label>
              <input readOnly value={licenseStatus.technical_message || 'OK'} />
            </div>
            <div>
              <label>Empresa bloqueada</label>
              <input readOnly value={licenseStatus.licensed_company?.bloqueado || licenseStatus.licensed_company?.bloqueio_admin ? 'Sim' : 'Não'} />
            </div>
            <div>
              <label>Máquina cadastrada</label>
              <input readOnly value={licenseStatus.machine_registered ? 'Sim' : 'Não'} />
            </div>
            <div>
              <label>Máquina bloqueada</label>
              <input readOnly value={licenseStatus.machine_blocked ? 'Sim' : 'Não'} />
            </div>
            <div>
              <label>Módulos remotos</label>
              <input readOnly value={licenseStatus.licensed_device?.modulos || licenseStatus.local_license?.licencas || 'Não informado'} />
            </div>
          </div>
        )}
      </div>

      <div className="card compact-card">
        <h3>Configuração única de empresa</h3>
        <p className="muted">
          A empresa utiliza um único layout de NFS-e, definido pelo município cadastrado.
        </p>
      </div>

      <ProfileForm value={profile} onChange={setProfile} />
      <FieldRuleEditor value={profile.field_rules} onChange={(rules) => setProfile({ ...profile, field_rules: rules })} />
    </div>
  );
}
