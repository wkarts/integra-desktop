import { useEffect, useMemo, useState } from 'react';
import type {
  LicenseRuntimeStatus,
  LicenseSettings,
  RegistrationDeviceInfo,
} from '../../../shared/types';
import {
  checkLicenseStatus,
  getAppMeta,
  getRegistrationDeviceInfo,
  loadLicenseSettings,
  saveLicenseSettings,
} from '../../nfse-servicos/services/tauriService';

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

const emptyDeviceInfo: RegistrationDeviceInfo = {
  station_name: '',
  device_display_name: '',
  hostname: '',
  computer_name: '',
  serial_number: '',
  machine_guid: '',
  bios_serial: '',
  motherboard_serial: '',
  logged_user: '',
  os_name: '',
  os_version: '',
  os_arch: '',
  domain_name: '',
  install_mode: '',
  mac_addresses: [],
  device_key: '',
  registration_file_found: false,
  registration_file_path: null,
  registration_file_verified: null,
};

export function StartupRegistrationGate() {
  const [booting, setBooting] = useState(true);
  const [required, setRequired] = useState(false);
  const [busy, setBusy] = useState(false);
  const [settings, setSettings] = useState<LicenseSettings>(defaultLicenseSettings);
  const [result, setResult] = useState<LicenseRuntimeStatus | null>(null);
  const [deviceInfo, setDeviceInfo] = useState<RegistrationDeviceInfo>(emptyDeviceInfo);
  const [error, setError] = useState('');
  const [bootMessage, setBootMessage] = useState('');

  useEffect(() => {
    async function bootstrap() {
      try {
        const savedSettings = await loadLicenseSettings();
        const nextSettings: LicenseSettings = {
          ...defaultLicenseSettings,
          ...savedSettings,
          auto_register_machine: true,
        };

        const device = await getRegistrationDeviceInfo(nextSettings);
        const meta = await getAppMeta();
        const hydratedSettings: LicenseSettings = {
          ...nextSettings,
          machine_key: nextSettings.machine_key || device.device_key,
          station_name: nextSettings.station_name || device.station_name,
        };

        setDeviceInfo(device);
        setSettings(hydratedSettings);

        const hasUserInput = Boolean(
          hydratedSettings.company_name.trim() ||
            hydratedSettings.company_document.trim() ||
            hydratedSettings.company_email.trim(),
        );
        const mayAutoRegister = device.registration_file_found || hasUserInput;

        if (mayAutoRegister) {
          const status = await checkLicenseStatus(hydratedSettings);
          setResult(status);

          if (status.allowed && status.machine_registered) {
            setRequired(false);
            return;
          }

          setBootMessage(status.message || 'Não foi possível concluir o registro automático.');
          setRequired(true);
          if (!hasUserInput && !device.registration_file_found) {
            setError('Informe a empresa licenciada para concluir a ativação desta instalação.');
          }
          return;
        }

        setBootMessage(
          `Informe a empresa licenciada do ${meta.product_name}. O dispositivo será cadastrado automaticamente.`,
        );
        setRequired(true);
      } catch (err) {
        setRequired(true);
        setError(err instanceof Error ? err.message : 'Falha ao preparar o registro inicial.');
      } finally {
        setBooting(false);
      }
    }

    void bootstrap();
  }, []);

  const canSubmit = useMemo(() => {
    if (busy) {
      return false;
    }

    if (deviceInfo.registration_file_found) {
      return true;
    }

    return Boolean(
      settings.company_document.trim() || settings.company_name.trim() || settings.company_email.trim(),
    );
  }, [busy, deviceInfo.registration_file_found, settings.company_document, settings.company_email, settings.company_name]);

  async function handleSubmit() {
    setBusy(true);
    setError('');
    try {
      const persisted = await saveLicenseSettings({
        ...settings,
        auto_register_machine: true,
      });
      setSettings(persisted);

      const status = await checkLicenseStatus(persisted);
      setResult(status);

      if (status.allowed && status.machine_registered) {
        setRequired(false);
        return;
      }

      setError(status.message || 'Não foi possível concluir o registro inicial da aplicação.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Falha ao registrar a aplicação.');
    } finally {
      setBusy(false);
    }
  }

  if (booting || !required) {
    return null;
  }

  return (
    <div className="startup-gate-overlay">
      <div className="startup-gate-card card">
        <h2>Registro inicial da aplicação</h2>
        <p className="muted">
          {bootMessage || 'Informe a empresa licenciada. O nome da estação já foi capturado automaticamente e o sistema vai registrar esta instalação na licença disponível.'}
        </p>

        {deviceInfo.registration_file_found && (
          <div className="alert-strip startup-gate-info">
            Arquivo/certificado local de registro localizado automaticamente.
            {deviceInfo.registration_file_path ? ` Caminho: ${deviceInfo.registration_file_path}` : ''}
            {typeof deviceInfo.registration_file_verified === 'boolean'
              ? ` | Assinatura ${deviceInfo.registration_file_verified ? 'válida' : 'não validada'}`
              : ''}
          </div>
        )}

        <div className="form-grid cols-4">
          <div>
            <label>Razão social licenciada</label>
            <input
              value={settings.company_name}
              onChange={(e) => setSettings({ ...settings, company_name: e.target.value })}
              placeholder="Empresa licenciada"
            />
          </div>
          <div>
            <label>CNPJ/CPF licenciado</label>
            <input
              value={settings.company_document}
              onChange={(e) => setSettings({ ...settings, company_document: e.target.value })}
              placeholder="Documento da empresa"
            />
          </div>
          <div>
            <label>E-mail</label>
            <input
              value={settings.company_email}
              onChange={(e) => setSettings({ ...settings, company_email: e.target.value })}
              placeholder="E-mail da empresa"
            />
          </div>
          <div>
            <label>Nome da estação</label>
            <input value={settings.station_name || deviceInfo.station_name} readOnly />
          </div>
          <div className="span-2">
            <label>Nome completo do dispositivo</label>
            <textarea
              value={deviceInfo.device_display_name || settings.station_name}
              readOnly
              rows={2}
              className="readonly-textarea mono-text"
            />
          </div>
          <div className="span-2">
            <label>Número de série completo</label>
            <textarea
              value={deviceInfo.serial_number || 'Não identificado automaticamente'}
              readOnly
              rows={2}
              className="readonly-textarea mono-text"
            />
          </div>
          <div>
            <label>Chave da máquina</label>
            <textarea
              value={settings.machine_key || deviceInfo.device_key}
              readOnly
              rows={2}
              className="readonly-textarea mono-text"
            />
          </div>
          <div>
            <label>Usuário logado</label>
            <input value={deviceInfo.logged_user || 'Não identificado'} readOnly />
          </div>
          <div>
            <label>Sistema operacional</label>
            <input
              value={[deviceInfo.os_name, deviceInfo.os_version, deviceInfo.os_arch].filter(Boolean).join(' | ')}
              readOnly
            />
          </div>
          <div>
            <label>Modo de instalação</label>
            <input value={deviceInfo.install_mode || 'workstation'} readOnly />
          </div>
        </div>

        {result && (
          <div className="status-panel">
            <div className="status-item">
              <span>Status</span>
              <strong>{result.allowed ? 'Liberada' : 'Bloqueada'}</strong>
            </div>
            <div className="status-item">
              <span>Máquinas liberadas</span>
              <strong>{result.seats_total}</strong>
            </div>
            <div className="status-item">
              <span>Máquinas em uso</span>
              <strong>{result.seats_used}</strong>
            </div>
            <div className="status-item">
              <span>Retorno</span>
              <strong>{result.message}</strong>
            </div>
          </div>
        )}

        {error && <div className="alert-strip startup-gate-error">{error}</div>}

        <div className="actions-row">
          <button className="btn primary" onClick={handleSubmit} disabled={!canSubmit}>
            {busy ? 'Registrando...' : 'Registrar aplicação'}
          </button>
        </div>
      </div>
    </div>
  );
}
