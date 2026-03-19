import type { NfseDocument } from '../../../shared/types';

interface DocsGridProps {
  documents: NfseDocument[];
  onDocumentsChange: (documents: NfseDocument[]) => void;
}

export function DocsGrid({ documents, onDocumentsChange }: DocsGridProps) {
  function update(index: number, path: string, rawValue: string) {
    const next = [...documents];
    const item = structuredClone(next[index]);
    const targetValue = path.startsWith('taxes.') ? Number(rawValue || 0) : rawValue;

    if (path === 'taxes.iss_retido') {
      item.taxes.iss_retido = rawValue === '1';
    } else {
      const [root, key] = path.split('.');
      if (root === 'taxes') {
        (item.taxes as unknown as Record<string, unknown>)[key] = targetValue;
      } else if (root === 'prestador') {
        (item.prestador as unknown as Record<string, unknown>)[key] = rawValue;
      } else if (root === 'tomador') {
        (item.tomador as unknown as Record<string, unknown>)[key] = rawValue;
      } else {
        (item as unknown as Record<string, unknown>)[path] = rawValue;
      }
    }

    next[index] = item;
    onDocumentsChange(next);
  }

  if (!documents.length) {
    return <div className="empty-box">Nenhum XML processado ainda.</div>;
  }

  return (
    <div className="card">
      <h3>Grid editável</h3>
      <div className="table-wrap">
        <table className="grid-table">
          <thead>
            <tr>
              <th>#</th>
              <th>Provider</th>
              <th>Número</th>
              <th>Série</th>
              <th>Emissão</th>
              <th>Município</th>
              <th>Cód. município</th>
              <th>Tomador</th>
              <th>Doc. tomador</th>
              <th>Serviço</th>
              <th>Valor serviços</th>
              <th>Base cálculo</th>
              <th>ISS</th>
              <th>Aliq.</th>
              <th>ISS retido</th>
              <th>IRRF</th>
              <th>PIS</th>
              <th>COFINS</th>
              <th>CSLL</th>
              <th>INSS</th>
              <th>Valor líquido</th>
              <th>Observação</th>
            </tr>
          </thead>
          <tbody>
            {documents.map((document, index) => (
              <tr key={document.id}>
                <td>{index + 1}</td>
                <td>{document.provider_friendly}</td>
                <td><input value={document.numero} onChange={(e) => update(index, 'numero', e.target.value)} /></td>
                <td><input value={document.serie} onChange={(e) => update(index, 'serie', e.target.value)} /></td>
                <td><input value={document.emissao} onChange={(e) => update(index, 'emissao', e.target.value)} /></td>
                <td><input value={document.municipio_nome} onChange={(e) => update(index, 'municipio_nome', e.target.value)} /></td>
                <td><input value={document.municipio_codigo} onChange={(e) => update(index, 'municipio_codigo', e.target.value)} /></td>
                <td><input value={document.tomador.nome} onChange={(e) => update(index, 'tomador.nome', e.target.value)} /></td>
                <td><input value={document.tomador.documento} onChange={(e) => update(index, 'tomador.documento', e.target.value)} /></td>
                <td><input value={document.item_lista_servico} onChange={(e) => update(index, 'item_lista_servico', e.target.value)} /></td>
                <td><input value={document.taxes.valor_servicos} onChange={(e) => update(index, 'taxes.valor_servicos', e.target.value)} /></td>
                <td><input value={document.taxes.base_calculo} onChange={(e) => update(index, 'taxes.base_calculo', e.target.value)} /></td>
                <td><input value={document.taxes.valor_iss} onChange={(e) => update(index, 'taxes.valor_iss', e.target.value)} /></td>
                <td><input value={document.taxes.aliquota_iss} onChange={(e) => update(index, 'taxes.aliquota_iss', e.target.value)} /></td>
                <td>
                  <select value={document.taxes.iss_retido ? '1' : '0'} onChange={(e) => update(index, 'taxes.iss_retido', e.target.value)}>
                    <option value="0">Não</option>
                    <option value="1">Sim</option>
                  </select>
                </td>
                <td><input value={document.taxes.valor_irrf} onChange={(e) => update(index, 'taxes.valor_irrf', e.target.value)} /></td>
                <td><input value={document.taxes.valor_pis} onChange={(e) => update(index, 'taxes.valor_pis', e.target.value)} /></td>
                <td><input value={document.taxes.valor_cofins} onChange={(e) => update(index, 'taxes.valor_cofins', e.target.value)} /></td>
                <td><input value={document.taxes.valor_csll} onChange={(e) => update(index, 'taxes.valor_csll', e.target.value)} /></td>
                <td><input value={document.taxes.valor_inss} onChange={(e) => update(index, 'taxes.valor_inss', e.target.value)} /></td>
                <td><input value={document.taxes.valor_liquido} onChange={(e) => update(index, 'taxes.valor_liquido', e.target.value)} /></td>
                <td><textarea value={document.discriminacao} onChange={(e) => update(index, 'discriminacao', e.target.value)} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
