import { PageHeader } from '../../../shared/components/PageHeader';
import integraLegacyUrl from '../../../assets/legacy/integra_legacy.html?url';
import manualLegacyUrl from '../../../assets/legacy/manual_faturas.html?url';

export default function LegadoPage() {
  return (
    <div className="stack-lg">
      <PageHeader title="Legado" subtitle="Fallback preservado para continuidade operacional, isolado da navegação principal." />
      <div className="grid-two">
        <div className="card frame-card">
          <h3>Integra legado</h3>
          <iframe title="Integra Legado" src={integraLegacyUrl} />
        </div>
        <div className="card frame-card">
          <h3>Manual legado</h3>
          <iframe title="Manual legado" src={manualLegacyUrl} />
        </div>
      </div>
    </div>
  );
}
