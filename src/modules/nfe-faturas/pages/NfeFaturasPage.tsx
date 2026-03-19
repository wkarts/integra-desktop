import { PageHeader } from '../../../shared/components/PageHeader';
import faturasLegacyUrl from '../../../assets/legacy/faturas_nfe_sped.html?url';

export default function NfeFaturasPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="NFe / Faturas" subtitle="Módulo legado operacional para processamento de duplicatas e SPED." />
      <div className="card frame-card">
        <iframe title="NFe / Faturas legado" src={faturasLegacyUrl} />
      </div>
    </div>
  );
}
