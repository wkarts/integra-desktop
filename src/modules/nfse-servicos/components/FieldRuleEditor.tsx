import type { ConversionFieldRules, FieldAction, FieldRule } from '../../../shared/types';

const labels: Record<keyof ConversionFieldRules, string> = {
  base_calculo: 'Base de cálculo',
  iss_aliquota: 'Alíquota ISS',
  iss_devido: 'ISS devido',
  iss_retido: 'ISS retido',
  valor_iss: 'Valor do ISS',
  valor_liquido: 'Valor líquido',
  valor_irrf: 'IRRF',
  valor_inss: 'INSS',
  valor_pis: 'PIS',
  valor_cofins: 'COFINS',
  valor_csll: 'CSLL',
  descontos: 'Descontos',
  deducoes: 'Deduções',
  observacao: 'Observação',
  codigo_servico: 'Código do serviço',
  municipio: 'Município',
  serie: 'Série',
  numero: 'Número',
  data_emissao: 'Data emissão',
  data_competencia: 'Data competência',
  tipo_documento: 'Tipo documento',
  especie_documento: 'Espécie documento',
  natureza_operacao: 'Natureza operação',
  campos_complementares: 'Campos complementares',
};

const actions: Array<{ value: FieldAction; label: string }> = [
  { value: 'source', label: 'Usar XML' },
  { value: 'zero', label: 'Zerar' },
  { value: 'empty', label: 'Anular / vazio' },
  { value: 'ignore', label: 'Ignorar' },
  { value: 'constant', label: 'Valor fixo' },
];

export function FieldRuleEditor({ value, onChange }: { value: ConversionFieldRules; onChange: (rules: ConversionFieldRules) => void }) {
  function updateRule(field: keyof ConversionFieldRules, next: FieldRule) {
    onChange({ ...value, [field]: next });
  }

  return (
    <div className="card">
      <h3>Regras de conversão por campo</h3>
      <p className="muted">
        Configure priorização de campos fiscais com uso direto do XML, zero, vazio, ignorado ou valor fixo persistido por perfil.
      </p>
      <div className="rules-grid">
        {Object.entries(value).map(([field, rule]) => {
          const typedField = field as keyof ConversionFieldRules;
          return (
            <div className="rule-box" key={field}>
              <label>{labels[typedField]}</label>
              <select
                value={rule.action}
                onChange={(event) => updateRule(typedField, { ...rule, action: event.target.value as FieldAction })}
              >
                {actions.map((action) => (
                  <option key={action.value} value={action.value}>{action.label}</option>
                ))}
              </select>
              {rule.action === 'constant' ? (
                <input
                  type="text"
                  value={rule.value ?? ''}
                  onChange={(event) => updateRule(typedField, { ...rule, value: event.target.value })}
                  placeholder="Valor fixo"
                />
              ) : null}
            </div>
          );
        })}
      </div>
    </div>
  );
}
