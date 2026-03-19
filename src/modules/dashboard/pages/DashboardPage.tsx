import { PageHeader } from '../../../shared/components/PageHeader';

export default function DashboardPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="Painel operacional" subtitle="Acompanhe processamento, exportações e atalhos de entrada fiscal." />
      <div className="kpi-grid">
        <div className="card kpi-card"><span>Atalho principal</span><strong>Importar NFS-e</strong><p className="muted">XML, ZIP e pasta com validação por arquivo.</p></div>
        <div className="card kpi-card"><span>Fallback ativo</span><strong>NFe / Faturas legado</strong><p className="muted">Fluxo isolado para evitar interrupção da operação.</p></div>
        <div className="card kpi-card"><span>Exportação</span><strong>TXT Prosoft + CSV</strong><p className="muted">Saída em lote com regras por campo no perfil.</p></div>
      </div>

      <div className="card">
        <h3>Como operar</h3>
        <ol className="clean-list">
          <li>Acesse <b>NFS-e → Prosoft</b> e importe XML, ZIP ou pasta.</li>
          <li>Revise documentos na grade, ajuste regras e confirme o perfil.</li>
          <li>Exporte TXT/CSV e acompanhe logs para inconsistências.</li>
        </ol>
      </div>
    </div>
  );
}
