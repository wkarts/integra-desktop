import type { ConversionFieldRules, FieldAction, FieldRule } from '../../../shared/types';

const labels: Record<keyof ConversionFieldRules, string> = {
  base_calculo: 'Base de cálculo',
  iss_aliquota: 'Alíquota ISS',
  iss_devido: 'ISS devido',
  iss_retido: 'ISS retido',
  valor_liquido: 'Valor líquido',
  valor_irrf: 'IRRF',
  valor_inss: 'INSS',
  valor_pis: 'PIS',
  valor_cofins: 'COFINS',
  valor_csll: 'CSLL',
  observacao: 'Observação',
  codigo_servico: 'Código do serviço',
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
        Você pode manter o valor do XML, zerar, anular em branco, ignorar ou forçar um valor fixo para campos como ISS, base de cálculo,
        retenções e observação.
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
