import { PageHeader } from '../../../shared/components/PageHeader';
import integraLegacyUrl from '../../../assets/legacy/integra_legacy.html?url';

export default function LegadoPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="Legado" subtitle="Único fallback preservado para fluxos ainda não migrados para a interface nova." />
      <div className="card frame-card">
        <iframe title="Integra legado" src={integraLegacyUrl} />
      </div>
    </div>
  );
}
