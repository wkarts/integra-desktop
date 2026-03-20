import { PageHeader } from '../../../shared/components/PageHeader';

export default function NfeFaturasPage() {
  return (
    <div className="stack-lg">
      <PageHeader
        title="NFe / Faturas"
        subtitle="Versão Tauri nativa do módulo, sem iframe legado e pronta para evolução incremental."
      />

      <div className="grid-two dashboard-grid">
        <div className="card">
          <h3>Status de migração</h3>
          <p className="muted">
            A camada HTML antiga foi removida. Este módulo agora roda no shell React + Tauri, mantendo o fluxo de
            licenciamento e observabilidade centralizados.
          </p>
          <ul className="clean-list">
            <li>Sem dependência de páginas legadas embarcadas.</li>
            <li>Componente pronto para conectar comandos Rust de importação/exportação.</li>
            <li>Mesma base visual responsiva da aplicação principal.</li>
          </ul>
        </div>

        <div className="card">
          <h3>Próximos passos técnicos</h3>
          <ol className="clean-list">
            <li>Implementar comandos Tauri específicos para NFe/Faturas.</li>
            <li>Persistir presets por perfil com trilha de auditoria em log local.</li>
            <li>Publicar exportação idempotente (deduplicação por hash do documento).</li>
          </ol>
        </div>
      </div>
    </div>
  );
}
