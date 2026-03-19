import type { NfseDocument } from '../../../shared/types';
import { moneyPtBr } from '../../../shared/utils/format';

export function StatsCards({ documents }: { documents: NfseDocument[] }) {
  const webiss = documents.filter((item) => item.provider.includes('webiss') || item.provider.includes('abrasf')).length;
  const ubaira = documents.filter((item) => item.provider.includes('ubaira')).length;
  const total = documents.reduce((acc, item) => acc + item.taxes.valor_servicos, 0);

  return (
    <div className="stats-row">
      <div className="stat-card"><span>Documentos</span><strong>{documents.length}</strong></div>
      <div className="stat-card"><span>WebISS / SAJ</span><strong>{webiss}</strong></div>
      <div className="stat-card"><span>Ubaíra</span><strong>{ubaira}</strong></div>
      <div className="stat-card"><span>Total serviços</span><strong>R$ {moneyPtBr(total)}</strong></div>
    </div>
  );
}
