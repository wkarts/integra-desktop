import { useEffect, useMemo, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../../nfse-servicos/stores/NfseStore';
import { ProfileForm } from '../../nfse-servicos/components/ProfileForm';
import { FieldRuleEditor } from '../../nfse-servicos/components/FieldRuleEditor';
import {
  checkLicenseStatus,
  getAppMeta,
  getDefaultStationName,
  getMachineFingerprint,
  loadLicenseSettings,
  loadProfileBundle,
  saveLicenseSettings,
  saveProfileBundle,
} from '../../nfse-servicos/services/tauriService';
import type {
  AppMeta,
  ConversionProfile,
  LicenseRuntimeStatus,
  LicenseSettings,
  ProfileBundle,
} from '../../../shared/types';
import { defaultProfile, defaultProfileBundle } from '../../../shared/mappers/defaultProfile';
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
  licensing_disabled: false,
};

const defaultMeta: AppMeta = {
  product_name: 'Integra Desktop',
  version: '1.6.0',
  build_hash: 'dev-local',
  app_id: 'integra-desktop',
};

function slugifyProfileId(value: string) {
  const normalized = value
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  return normalized || `perfil-${Date.now()}`;
}

function getSelectedProfile(bundle: ProfileBundle): ConversionProfile {
  return (
    bundle.profiles.find((item) => item.profile_id === bundle.selected_profile_id) ||
    bundle.profiles[0] ||
    defaultProfile
  );
}

export default function SettingsPage() {
  const { profile, setProfile, pushLog } = useNfseStore();
  const [licenseSettings, setLicenseSettings] = useState<LicenseSettings>(defaultLicenseSettings);
  const [licenseStatus, setLicenseStatus] = useState<LicenseRuntimeStatus | null>(null);
  const [meta, setMeta] = useState<AppMeta>(defaultMeta);
  const [profileBundle, setProfileBundle] = useState<ProfileBundle>(defaultProfileBundle);
  const [busyLicense, setBusyLicense] = useState(false);
  const [busyProfiles, setBusyProfiles] = useState(false);

  useEffect(() => {
    Promise.all([
      loadLicenseSettings(),
      getMachineFingerprint(),
      getDefaultStationName(),
      loadProfileBundle(),
      getAppMeta(),
    ])
      .then(([savedLicense, fingerprint, stationName, savedBundle, appMeta]) => {
        setMeta(appMeta);
        setLicenseSettings(
          savedLicense ?? {
            ...defaultLicenseSettings,
            machine_key: fingerprint,
            station_name: stationName,
          },
        );

        const nextBundle = savedBundle ?? defaultProfileBundle;
        setProfileBundle(nextBundle);
        setProfile(getSelectedProfile(nextBundle));
      })
      .catch(() => pushLog('Falha ao carregar configurações locais.'));
  }, [pushLog, setProfile]);

  const selectedProfile = useMemo(
    () => getSelectedProfile(profileBundle),
    [profileBundle],
  );

  useEffect(() => {
    if (profile.profile_id !== selectedProfile.profile_id) {
      setProfile(selectedProfile);
    }
  }, [profile.profile_id, selectedProfile, setProfile]);

  function updateSelectedProfile(nextProfile: ConversionProfile) {
    setProfile(nextProfile);
    setProfileBundle((current) => {
      const exists = current.profiles.some((item) => item.profile_id === nextProfile.profile_id);
      const profiles = exists
        ? current.profiles.map((item) =>
            item.profile_id === nextProfile.profile_id ? nextProfile : item,
          )
        : [...current.profiles, nextProfile];

      return {
        selected_profile_id: nextProfile.profile_id,
        profiles,
      };
    });
  }

  function handleSelectProfile(profileId: string) {
    setProfileBundle((current) => ({
      ...current,
      selected_profile_id: profileId,
    }));

    const next = profileBundle.profiles.find((item) => item.profile_id === profileId);
    if (next) {
      setProfile(next);
    }
  }

  function handleCreateProfile() {
    const baseName = `Novo perfil ${profileBundle.profiles.length + 1}`;
    const profileId = slugifyProfileId(baseName);
    const nextProfile: ConversionProfile = {
      ...defaultProfile,
      ...selectedProfile,
      profile_id: profileId,
      profile_name: baseName,
      profile_company_name: '',
      profile_company_document: '',
    };

    updateSelectedProfile(nextProfile);
    pushLog(`Novo perfil criado: ${baseName}.`);
  }

  function handleDuplicateProfile() {
    const duplicatedName = `${selectedProfile.profile_name || 'Perfil'} (cópia)`;
    const nextProfile: ConversionProfile = {
      ...selectedProfile,
      profile_id: `${slugifyProfileId(selectedProfile.profile_name)}-${Date.now()}`,
      profile_name: duplicatedName,
    };

    updateSelectedProfile(nextProfile);
    pushLog(`Perfil duplicado: ${duplicatedName}.`);
  }

  function handleRemoveProfile() {
    if (profileBundle.profiles.length <= 1) {
      pushLog('Mantenha ao menos um perfil de empresa para conversão.');
      return;
    }

    const remaining = profileBundle.profiles.filter(
      (item) => item.profile_id !== selectedProfile.profile_id,
    );
    const fallback = remaining[0] || defaultProfile;

    const nextBundle: ProfileBundle = {
      selected_profile_id: fallback.profile_id,
      profiles: remaining,
    };

    setProfileBundle(nextBundle);
    setProfile(fallback);
    pushLog(`Perfil removido: ${selectedProfile.profile_name || selectedProfile.profile_id}.`);
  }

  async function handleSaveLicenseSettings() {
    setBusyLicense(true);
    try {
      const persisted = await saveLicenseSettings({
        ...licenseSettings,
        auto_register_machine: true,
      });
      setLicenseSettings(persisted);
      pushLog('Configurações de licenciamento da aplicação salvas com sucesso.');
    } finally {
      setBusyLicense(false);
    }
  }

  async function handleCheckLicense() {
    setBusyLicense(true);
    try {
      const persisted = await saveLicenseSettings({
        ...licenseSettings,
        auto_register_machine: true,
      });
      setLicenseSettings(persisted);

      const result = await checkLicenseStatus(persisted);
      setLicenseStatus(result);

      if (result.allowed && result.machine_registered) {
        pushLog('Licenciamento da aplicação validado e estação registrada com sucesso.');
      } else {
        pushLog(
          `Licenciamento bloqueado: ${result.message} (${result.block_reason ?? 'sem motivo informado'})`,
        );
      }
    } finally {
      setBusyLicense(false);
    }
  }

  async function handleSaveProfiles() {
    const issues = validateProfile(selectedProfile);
    if (issues.length) {
      issues.forEach((issue) => pushLog(`Validação do perfil: ${issue}`));
      return;
    }

    setBusyProfiles(true);
    try {
      const normalizedBundle: ProfileBundle = {
        selected_profile_id: selectedProfile.profile_id,
        profiles: profileBundle.profiles.map((item) =>
          item.profile_id === selectedProfile.profile_id ? selectedProfile : item,
        ),
      };

      await saveProfileBundle(normalizedBundle);
      setProfileBundle(normalizedBundle);
      pushLog(
        `Perfis de empresa salvos com sucesso (${normalizedBundle.profiles.length} perfil(is)).`,
      );
    } finally {
      setBusyProfiles(false);
    }
  }

  return (
    <div className="stack-lg">
      <PageHeader
        title="Configurações"
        subtitle="Licenciamento da aplicação e perfis de empresa separados corretamente."
        actions={(
          <div className="actions-row">
            <button className="btn" onClick={handleCheckLicense} disabled={busyLicense}>
              Validar e registrar aplicação
            </button>
            <button className="btn primary" onClick={handleSaveLicenseSettings} disabled={busyLicense}>
              Salvar licença
            </button>
            <button className="btn success" onClick={handleSaveProfiles} disabled={busyProfiles}>
              Salvar perfis
            </button>
          </div>
        )}
      />

      <div className="card">
        <div className="section-title-row">
          <div>
            <h3>Licenciamento da aplicação</h3>
            <p className="muted">
              Controle da licença do software e da estação atual, sem interferir nos perfis de empresa.
            </p>
          </div>
          <div className="meta-badges">
            <span className="badge">Versão {meta.version}</span>
            <span className="badge">ASHA {meta.build_hash.slice(0, 12)}</span>
          </div>
        </div>

        <div className="form-grid cols-4">
          <div>
            <label>Razão social licenciada</label>
            <input
              value={licenseSettings.company_name}
              onChange={(e) =>
                setLicenseSettings({ ...licenseSettings, company_name: e.target.value })
              }
            />
          </div>
          <div>
            <label>CNPJ/CPF licenciado</label>
            <input
              value={licenseSettings.company_document}
              onChange={(e) =>
                setLicenseSettings({ ...licenseSettings, company_document: e.target.value })
              }
            />
          </div>
          <div>
            <label>E-mail</label>
            <input
              value={licenseSettings.company_email}
              onChange={(e) =>
                setLicenseSettings({ ...licenseSettings, company_email: e.target.value })
              }
            />
          </div>
          <div>
            <label>Nome da estação</label>
            <input value={licenseSettings.station_name} readOnly />
          </div>
          <div>
            <label>Chave da máquina</label>
            <input value={licenseSettings.machine_key} readOnly />
          </div>
          <div>
            <label>Registro automático</label>
            <input value={licenseSettings.auto_register_machine ? 'Ativado' : 'Desativado'} readOnly />
          </div>
        </div>

        <div className="status-panel">
          <div className="status-item">
            <span>Status</span>
            <strong>{licenseStatus ? (licenseStatus.allowed ? 'Liberada' : 'Bloqueada') : 'Não verificada'}</strong>
          </div>
          <div className="status-item">
            <span>Máquinas liberadas</span>
            <strong>{licenseStatus?.seats_total ?? 0}</strong>
          </div>
          <div className="status-item">
            <span>Máquinas em uso</span>
            <strong>{licenseStatus?.seats_used ?? 0}</strong>
          </div>
          <div className="status-item">
            <span>Retorno</span>
            <strong>{licenseStatus?.message || 'Ainda não consultado'}</strong>
          </div>
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
              <input
                readOnly
                value={
                  licenseStatus.licensed_company?.bloqueado ||
                  licenseStatus.licensed_company?.bloqueio_admin
                    ? 'Sim'
                    : 'Não'
                }
              />
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
              <input
                readOnly
                value={
                  licenseStatus.licensed_device?.modulos ||
                  licenseStatus.local_license?.licencas ||
                  'Não informado'
                }
              />
            </div>
          </div>
        )}
      </div>

      <div className="card">
        <div className="section-title-row profile-toolbar">
          <div>
            <h3>Perfis de empresa para importação/exportação</h3>
            <p className="muted">
              Esses perfis são operacionais e independem da licença da aplicação.
            </p>
          </div>
          <div className="profile-toolbar-actions">
            <select
              value={selectedProfile.profile_id}
              onChange={(e) => handleSelectProfile(e.target.value)}
            >
              {profileBundle.profiles.map((item) => (
                <option key={item.profile_id} value={item.profile_id}>
                  {item.profile_name || item.profile_company_name || item.profile_id}
                </option>
              ))}
            </select>
            <button className="btn" onClick={handleCreateProfile} disabled={busyProfiles}>
              Novo perfil
            </button>
            <button className="btn" onClick={handleDuplicateProfile} disabled={busyProfiles}>
              Duplicar
            </button>
            <button className="btn danger" onClick={handleRemoveProfile} disabled={busyProfiles}>
              Remover
            </button>
          </div>
        </div>

        <div className="alert-strip" style={{ marginBottom: 12 }}>
          Perfis cadastrados: <b>{profileBundle.profiles.length}</b>. Perfil ativo: <b>{selectedProfile.profile_name}</b>.
        </div>

        <ProfileForm value={selectedProfile} onChange={updateSelectedProfile} />
        <FieldRuleEditor
          value={selectedProfile.field_rules}
          onChange={(rules) => updateSelectedProfile({ ...selectedProfile, field_rules: rules })}
        />
      </div>
    </div>
  );
}
