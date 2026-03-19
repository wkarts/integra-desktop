import { PageHeader } from '../../../shared/components/PageHeader';

export default function LegadoPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="Legado" subtitle="Fallback preservado para fluxos já existentes. Mantido isolado para não bloquear a evolução da aplicação Tauri." />
      <div className="grid-two">
        <div className="card frame-card">
          <h3>HTML legado principal</h3>
          <iframe title="Integra Legado" src="./assets/legacy/integra_legacy.html" />
        </div>
        <div className="card frame-card">
          <h3>Manual legado</h3>
          <iframe title="Manual legado" src="./assets/legacy/manual_faturas.html" />
        </div>
      </div>
    </div>
  );
}
