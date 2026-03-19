import { useEffect, useState } from 'react';
import { PageHeader } from '../../../shared/components/PageHeader';
import { listLogs } from '../../nfse-servicos/services/tauriService';

export default function LogsPage() {
  const [logs, setLogs] = useState<string[]>([]);

  useEffect(() => {
    listLogs().then(setLogs).catch(() => setLogs(['Falha ao carregar logs do backend Tauri.']));
  }, []);

  return (
    <div className="stack-lg">
      <PageHeader title="Logs" subtitle="Log persistido no backend Tauri." />
      <div className="card">
        <pre className="console-box">{logs.join('\n') || '(sem logs persistidos)'}</pre>
      </div>
    </div>
  );
}
