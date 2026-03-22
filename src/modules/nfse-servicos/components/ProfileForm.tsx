import type { ConversionProfile } from '../../../shared/types';

export function ProfileForm({ value, onChange }: { value: ConversionProfile; onChange: (profile: ConversionProfile) => void }) {
  function set<K extends keyof ConversionProfile>(field: K, next: ConversionProfile[K]) {
    onChange({ ...value, [field]: next });
  }

  return (
    <div className="card">
      <h3>Perfil de conversão</h3>
      <div className="form-grid cols-4">
        <div>
          <label>Nome da empresa</label>
          <input value={value.profile_name} onChange={(e) => set('profile_name', e.target.value)} />
        </div>
        <div>
          <label>Razão social para escrituração</label>
          <input value={value.profile_company_name} onChange={(e) => set('profile_company_name', e.target.value)} placeholder="Ex.: Escrituração ACME Ltda" />
        </div>
        <div>
          <label>CNPJ/CPF da empresa</label>
          <input value={value.profile_company_document} onChange={(e) => set('profile_company_document', e.target.value)} />
        </div>
        <div>
          <label>Município da empresa</label>
          <input value={value.company_municipio_nome} onChange={(e) => set('company_municipio_nome', e.target.value)} placeholder="Ex.: Santo Antônio de Jesus" />
        </div>
        <div>
          <label>Código IBGE município</label>
          <input value={value.company_municipio_codigo} onChange={(e) => set('company_municipio_codigo', e.target.value)} placeholder="Opcional" />
        </div>
        <div>
          <label>Layout municipal NFS-e</label>
          <select value={value.nfse_layout} onChange={(e) => set('nfse_layout', e.target.value as ConversionProfile['nfse_layout'])}>
            <option value="auto">Automático (detecção)</option>
            <option value="webiss_abrasf_v2">ABRASF / WebISS</option>
            <option value="ginfes">GINFES / SAJ</option>
            <option value="betha">Betha</option>
            <option value="abrasf_v1">ABRASF v1</option>
            <option value="ubaira_custom">Ubaíra (custom)</option>
          </select>
        </div>
        <div>
          <label>Layout de saída</label>
          <select value={value.output_layout} onChange={(e) => set('output_layout', e.target.value as ConversionProfile['output_layout'])}>
            <option value="ba_prestados">Serviços prestados</option>
            <option value="ba_tomados">Serviços tomados</option>
            <option value="prosoft_faturas">Faturas</option>
          </select>
        </div>
        <div>
          <label>Empresa usuária</label>
          <input value={value.user_company_name} onChange={(e) => set('user_company_name', e.target.value)} placeholder="Empresa licenciada" />
        </div>
        <div>
          <label>CNPJ/CPF empresa usuária</label>
          <input value={value.user_company_document} onChange={(e) => set('user_company_document', e.target.value)} />
        </div>
        <div>
          <label>COD Prosoft</label>
          <input value={value.cod_prosoft} onChange={(e) => set('cod_prosoft', e.target.value)} />
        </div>
        <div>
          <label>Espécie documento</label>
          <input value={value.especie_documento} onChange={(e) => set('especie_documento', e.target.value)} />
        </div>
        <div>
          <label>Modelo NF</label>
          <input value={value.modelo_nf} onChange={(e) => set('modelo_nf', e.target.value)} />
        </div>
        <div>
          <label>Tipo documento</label>
          <input value={value.tipo_documento} onChange={(e) => set('tipo_documento', e.target.value)} />
        </div>
        <div>
          <label>Situação documento</label>
          <input value={value.situacao_documento} onChange={(e) => set('situacao_documento', e.target.value)} />
        </div>
        <div>
          <label>CFPS</label>
          <input value={value.cfps} onChange={(e) => set('cfps', e.target.value)} />
        </div>
        <div>
          <label>Responsável retenção</label>
          <select value={value.responsavel_retencao} onChange={(e) => set('responsavel_retencao', e.target.value)}>
            <option value="0">0 - Nenhum</option>
            <option value="1">1 - Contratante</option>
            <option value="2">2 - Subcontratante</option>
          </select>
        </div>
        <div>
          <label>Cód. rec. IRRF</label>
          <input value={value.cod_receita_irrf} onChange={(e) => set('cod_receita_irrf', e.target.value)} />
        </div>
        <div>
          <label>Cód. rec. PIS</label>
          <input value={value.cod_rec_pis} onChange={(e) => set('cod_rec_pis', e.target.value)} />
        </div>
        <div>
          <label>Cód. rec. COFINS</label>
          <input value={value.cod_rec_cofins} onChange={(e) => set('cod_rec_cofins', e.target.value)} />
        </div>
        <div>
          <label>Forçar *OBS</label>
          <select value={value.obs_extended} onChange={(e) => set('obs_extended', e.target.value as ConversionProfile['obs_extended'])}>
            <option value="auto">Automático</option>
            <option value="always">Sempre</option>
            <option value="never">Nunca</option>
          </select>
        </div>
      </div>
    </div>
  );
}
