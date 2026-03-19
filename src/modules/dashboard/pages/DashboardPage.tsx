import { PageHeader } from '../../../shared/components/PageHeader';

export default function DashboardPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="Dashboard" subtitle="Aplicação desktop fiscal construída em Tauri com núcleo Rust, frontend React/TypeScript e fallback legado HTML." />
      <div className="card">
        <h3>Escopo desta versão</h3>
        <ul className="clean-list">
          <li>Motor novo para NFS-e → Prosoft em Rust/Tauri.</li>
          <li>Fallback legado preservado para NFe / SPED / Faturas em HTML.</li>
          <li>Parâmetros por campo para zerar, anular em branco, ignorar ou fixar valores.</li>
          <li>Suporte inicial implementado para WebISS/SAJ e layout específico de Ubaíra/BA.</li>
        </ul>
      </div>
    </div>
  );
}
