import { useEffect, useMemo, useState } from 'react';
import type { LicenseRuntimeStatus, LicenseSettings } from '../../../shared/types';
import {
  checkLicenseStatus,
  getDefaultStationName,
  getMachineFingerprint,
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

export function StartupRegistrationGate() {
  const [booting, setBooting] = useState(true);
  const [required, setRequired] = useState(false);
  const [busy, setBusy] = useState(false);
  const [settings, setSettings] = useState<LicenseSettings>(defaultLicenseSettings);
  const [result, setResult] = useState<LicenseRuntimeStatus | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    Promise.all([
      loadLicenseSettings(),
      getMachineFingerprint(),
      getDefaultStationName(),
    ])
      .then(([savedSettings, machineKey, stationName]) => {
        const nextSettings = {
          ...defaultLicenseSettings,
          ...savedSettings,
          machine_key: savedSettings?.machine_key || machineKey,
          station_name: savedSettings?.station_name || stationName,
          auto_register_machine: true,
        };

        setSettings(nextSettings);
        setRequired(!nextSettings.company_name.trim() || !nextSettings.company_document.trim());
      })
      .catch(() => {
        setRequired(true);
      })
      .finally(() => setBooting(false));
  }, []);

  const canSubmit = useMemo(
    () => Boolean(settings.company_name.trim() && settings.company_document.trim()),
    [settings.company_document, settings.company_name],
  );

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
          Informe a empresa licenciada. O nome da estação já foi capturado automaticamente e o sistema vai registrar esta instalação na licença disponível.
        </p>

        <div className="form-grid cols-4">
          <div>
            <label>Razão social licenciada</label>
            <input
              value={settings.company_name}
              onChange={(e) => setSettings({ ...settings, company_name: e.target.value })}
            />
          </div>
          <div>
            <label>CNPJ/CPF licenciado</label>
            <input
              value={settings.company_document}
              onChange={(e) => setSettings({ ...settings, company_document: e.target.value })}
            />
          </div>
          <div>
            <label>E-mail</label>
            <input
              value={settings.company_email}
              onChange={(e) => setSettings({ ...settings, company_email: e.target.value })}
            />
          </div>
          <div>
            <label>Nome da estação</label>
            <input value={settings.station_name} readOnly />
          </div>
          <div>
            <label>Chave da máquina</label>
            <input value={settings.machine_key} readOnly />
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
          <button className="btn primary" onClick={handleSubmit} disabled={busy || !canSubmit}>
            {busy ? 'Registrando...' : 'Registrar aplicação'}
          </button>
        </div>
      </div>
    </div>
  );
}
