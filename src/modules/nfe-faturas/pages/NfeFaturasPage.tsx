import { PageHeader } from '../../../shared/components/PageHeader';

export default function NfeFaturasPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="NFe / Faturas" subtitle="Fallback operacional do recurso legado enquanto o motor novo avança por etapas." />
      <div className="card frame-card">
        <iframe title="NFe / Faturas legado" src="./assets/legacy/faturas_nfe_sped.html" />
      </div>
    </div>
  );
}
