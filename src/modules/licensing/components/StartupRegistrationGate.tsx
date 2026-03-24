import { useEffect, useMemo, useState } from 'react';
import type {
  LicenseRuntimeStatus,
  LicenseSettings,
  RegistrationDeviceInfo,
  StartupLicenseContext,
} from '../../../shared/types';
import {
  checkLicenseStatus,
  getAppMeta,
  getRegistrationDeviceInfo,
  getStartupLicensingContext,
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
  auto_register_requested_licenses: null,
  auto_register_validation_mode: 'standard',
  auto_register_interface_mode: 'interactive',
  auto_register_device_identifier: '',
};


const emptyStartupContext: StartupLicenseContext = {
  auto_register_enabled: false,
  auto_register_company: false,
  auto_register_device: false,
  requested_licenses: null,
  company_name: null,
  company_document: null,
  company_email: null,
  station_name: null,
  device_name: null,
  device_identifier: null,
  validation_mode: 'standard',
  interface_mode: 'interactive',
  local_license_enabled: false,
  local_license_generate: false,
  local_license_file_path: null,
  local_license_token_present: false,
  developer_secret_present: false,
  local_license_account: null,
  local_license_issuer: null,
  no_ui: false,
  args: [],
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
  const [startupContext, setStartupContext] = useState<StartupLicenseContext>(emptyStartupContext);
  const [error, setError] = useState('');
  const [bootMessage, setBootMessage] = useState('');

  useEffect(() => {
    async function bootstrap() {
      try {
        const savedSettings = await loadLicenseSettings();
        const cliContext = await getStartupLicensingContext();
        setStartupContext(cliContext);

        const nextSettings: LicenseSettings = {
          ...defaultLicenseSettings,
          ...savedSettings,
          auto_register_machine: cliContext.auto_register_enabled || true,
          company_name: cliContext.company_name || savedSettings?.company_name || '',
          company_document: cliContext.company_document || savedSettings?.company_document || '',
          company_email: cliContext.company_email || savedSettings?.company_email || '',
          station_name: cliContext.station_name || savedSettings?.station_name || '',
          auto_register_requested_licenses:
            cliContext.requested_licenses ?? savedSettings?.auto_register_requested_licenses ?? null,
          auto_register_validation_mode:
            cliContext.validation_mode || savedSettings?.auto_register_validation_mode || 'standard',
          auto_register_interface_mode:
            cliContext.interface_mode || savedSettings?.auto_register_interface_mode || 'interactive',
          auto_register_device_identifier:
            cliContext.device_identifier || savedSettings?.auto_register_device_identifier || '',
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
        const mayAutoRegister =
          cliContext.auto_register_enabled ||
          cliContext.local_license_enabled ||
          device.registration_file_found ||
          hasUserInput;

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
          cliContext.auto_register_enabled
            ? `Modo de auto-registro habilitado para o ${meta.product_name}. Complete os dados faltantes para concluir o cadastro automático.`
            : `Informe a empresa licenciada do ${meta.product_name}. O dispositivo será cadastrado automaticamente.`,
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

    if (deviceInfo.registration_file_found || startupContext.local_license_enabled) {
      return true;
    }

    return Boolean(
      settings.company_document.trim() || settings.company_name.trim() || settings.company_email.trim(),
    );
  }, [
    busy,
    deviceInfo.registration_file_found,
    settings.company_document,
    settings.company_email,
    settings.company_name,
    startupContext.local_license_enabled,
  ]);

  async function handleSubmit() {
    setBusy(true);
    setError('');
    try {
      const persisted = await saveLicenseSettings({
        ...settings,
        auto_register_machine: startupContext.auto_register_enabled || true,
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
        {startupContext.auto_register_enabled && (
          <div className="alert-strip startup-gate-info">
            Modo opcional de auto-registro habilitado por parâmetro de execução.
            {typeof startupContext.requested_licenses === 'number'
              ? ` Licenças solicitadas: ${startupContext.requested_licenses}.`
              : ''}
            {startupContext.validation_mode ? ` Validação: ${startupContext.validation_mode}.` : ''}
            {startupContext.interface_mode ? ` Interface: ${startupContext.interface_mode}.` : ''}
          </div>
        )}

        {startupContext.local_license_enabled && (
          <div className="alert-strip startup-gate-info">
            Modo opcional de licença local habilitado.
            {startupContext.local_license_file_path ? ` Arquivo: ${startupContext.local_license_file_path}.` : ''}
            {startupContext.local_license_generate ? ' Geração local solicitada.' : ' Validação local solicitada.'}
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
          <div>
            <label>Licenças a liberar (opcional)</label>
            <input
              type="number"
              min={0}
              value={settings.auto_register_requested_licenses ?? ''}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  auto_register_requested_licenses:
                    e.target.value.trim() === '' ? null : Number(e.target.value),
                })
              }
              placeholder="Ex.: 5"
            />
          </div>
          <div>
            <label>Modo de validação</label>
            <input
              value={settings.auto_register_validation_mode || startupContext.validation_mode || 'standard'}
              onChange={(e) =>
                setSettings({ ...settings, auto_register_validation_mode: e.target.value })
              }
              placeholder="standard | local-only | online-only"
            />
          </div>
          <div>
            <label>Comportamento da interface</label>
            <input
              value={settings.auto_register_interface_mode || startupContext.interface_mode || 'interactive'}
              onChange={(e) =>
                setSettings({ ...settings, auto_register_interface_mode: e.target.value })
              }
              placeholder="interactive | silent | assisted"
            />
          </div>
          <div>
            <label>Identificador do dispositivo</label>
            <input
              value={settings.auto_register_device_identifier || startupContext.device_identifier || ''}
              onChange={(e) =>
                setSettings({ ...settings, auto_register_device_identifier: e.target.value })
              }
              placeholder="Etiqueta, ativo ou código interno"
            />
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
