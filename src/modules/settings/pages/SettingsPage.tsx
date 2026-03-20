import { useEffect, useMemo, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../../nfse-servicos/stores/NfseStore';
import { ProfileForm } from '../../nfse-servicos/components/ProfileForm';
import { FieldRuleEditor } from '../../nfse-servicos/components/FieldRuleEditor';
import {
  checkLicenseStatus,
  getAppMeta,
  getMachineFingerprint,
  loadLicenseSettings,
  loadProfileBundle,
  saveLicenseSettings,
  saveProfileBundle,
} from '../../nfse-servicos/services/tauriService';
import type { AppMeta, LicenseRuntimeStatus, LicenseSettings, ProfileBundle } from '../../../shared/types';
import { defaultProfileBundle } from '../../../shared/mappers/defaultProfile';
import { validateProfile } from '../../../shared/validators/profiles';

const LICENSE_BASE_URL = 'https://api.rest.wwsoftwares.com.br';
const LICENSE_ENDPOINT = '/api/v1/';
const LICENSE_SERVICE_URL = `${LICENSE_BASE_URL}${LICENSE_ENDPOINT}`;
const LICENSE_APP_INSTANCE = 'integra-desktop';
const LICENSE_AUTO_REGISTER_MACHINE = true;

const defaultLicenseSettings: LicenseSettings = {
  service_url: LICENSE_SERVICE_URL,
  company_name: '',
  company_document: '',
  company_email: '',
  station_name: '',
  machine_key: '',
  auto_register_machine: LICENSE_AUTO_REGISTER_MACHINE,
  app_instance: LICENSE_APP_INSTANCE,
};

const defaultMeta: AppMeta = {
  product_name: 'Integra Desktop',
  version: '1.0.3',
  build_hash: 'dev-local',
  app_id: 'integra-desktop',
};

export default function SettingsPage() {
  const { profile, setProfile, pushLog } = useNfseStore();
  const [licenseSettings, setLicenseSettings] = useState<LicenseSettings>(defaultLicenseSettings);
  const [licenseStatus, setLicenseStatus] = useState<LicenseRuntimeStatus | null>(null);
  const [bundle, setBundle] = useState<ProfileBundle>(defaultProfileBundle);
  const [meta, setMeta] = useState<AppMeta>(defaultMeta);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    Promise.all([loadLicenseSettings(), getMachineFingerprint(), loadProfileBundle(), getAppMeta()])
      .then(([savedLicense, fingerprint, savedBundle, appMeta]) => {
        setMeta(appMeta);
        setLicenseSettings({
          ...defaultLicenseSettings,
          ...(savedLicense ?? {}),
          machine_key: savedLicense?.machine_key || fingerprint,
          service_url: LICENSE_SERVICE_URL,
          app_instance: LICENSE_APP_INSTANCE,
          auto_register_machine: LICENSE_AUTO_REGISTER_MACHINE,
        });
        const nextBundle = savedBundle ?? defaultProfileBundle;
        setBundle(nextBundle);
        const activeProfile = nextBundle.profiles.find((item) => item.profile_id === nextBundle.selected_profile_id) ?? nextBundle.profiles[0];
        if (activeProfile) setProfile(activeProfile);
      })
      .catch(() => pushLog('Falha ao carregar configurações locais.'));
  }, [pushLog, setProfile]);

  const profileNames = useMemo(() => bundle.profiles.map((item) => item.profile_company_name || item.profile_name), [bundle]);

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
        service_url: LICENSE_SERVICE_URL,
        app_instance: LICENSE_APP_INSTANCE,
        auto_register_machine: LICENSE_AUTO_REGISTER_MACHINE,
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

      const nextProfiles = bundle.profiles.some((item) => item.profile_id === nextProfile.profile_id)
        ? bundle.profiles.map((item) => item.profile_id === nextProfile.profile_id ? nextProfile : item)
        : [...bundle.profiles, nextProfile];

      const nextBundle = { selected_profile_id: nextProfile.profile_id, profiles: nextProfiles };
      await saveProfileBundle(nextBundle);
      setBundle(nextBundle);
      pushLog('Configurações gerais e perfis salvos com sucesso.');
    } finally {
      setBusy(false);
    }
  }

  async function handleCheckLicense() {
    setBusy(true);
    try {
      const result = await checkLicenseStatus({
        ...licenseSettings,
        service_url: LICENSE_SERVICE_URL,
        app_instance: LICENSE_APP_INSTANCE,
        auto_register_machine: LICENSE_AUTO_REGISTER_MACHINE,
      });
      setLicenseStatus(result);
      if (!result.allowed) {
        pushLog(`Licenciamento bloqueado: ${result.message} (${result.block_reason ?? 'sem motivo informado'})`);
      }
    } finally {
      setBusy(false);
    }
  }

  async function selectProfile(profileId: string) {
    const selected = bundle.profiles.find((item) => item.profile_id === profileId);
    if (!selected) return;
    setProfile(selected);
    const nextBundle = { ...bundle, selected_profile_id: profileId };
    setBundle(nextBundle);
    await saveProfileBundle(nextBundle);
    pushLog(`Perfil ativo nas configurações: ${selected.profile_company_name || selected.profile_name}.`);
  }

  return (
    <div className="stack-lg">
      <PageHeader
        title="Configurações"
        subtitle="Cadastre a Razão Social da empresa e gerencie perfis de escrituração com licenciamento transparente ao usuário final."
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
            <h3>Licenciamento</h3>
            <p className="muted">WS, instância e auto cadastro de estação ficam fixos internamente e não são expostos na interface.</p>
          </div>
          <div className="meta-badges">
            <span className="badge">Versão {meta.version}</span>
            <span className="badge">ASHA {meta.build_hash.slice(0, 12)}</span>
          </div>
        </div>

        <div className="form-grid cols-4">
          <div>
            <label>Razão Social</label>
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
          </div>
        )}
      </div>

      <div className="card compact-card">
        <h3>Perfis cadastrados</h3>
        <p className="muted">Cada perfil representa a empresa para a qual será feita a escrituração.</p>
        <div className="profile-toolbar-actions">
          <select value={profile.profile_id} onChange={(e) => void selectProfile(e.target.value)}>
            {bundle.profiles.map((item) => (
              <option key={item.profile_id} value={item.profile_id}>{item.profile_company_name || item.profile_name}</option>
            ))}
          </select>
          <span className="muted">Perfis disponíveis: {profileNames.join(', ') || 'nenhum'}</span>
        </div>
      </div>

      <ProfileForm value={profile} onChange={setProfile} />
      <FieldRuleEditor value={profile.field_rules} onChange={(rules) => setProfile({ ...profile, field_rules: rules })} />
    </div>
  );
}
