import { PageHeader } from '../../../shared/components/PageHeader';

export default function NfeFaturasPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="NFe / Faturas" subtitle="Versão desktop (Tauri) do fluxo de faturamento e conferência fiscal." />
      <div className="card">
        <h3>Módulo Tauri ativo</h3>
        <p className="muted">
          O conteúdo legado em HTML foi removido desta tela. Esta área agora permanece nativa, com
          comportamento consistente com o restante do aplicativo.
        </p>
      </div>
    </div>
  );
}
