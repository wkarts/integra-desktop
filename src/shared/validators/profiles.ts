import type { ConversionProfile } from '../types';

export function validateProfile(profile: ConversionProfile): string[] {
  const issues: string[] = [];
  if (!profile.profile_name.trim()) issues.push('Informe o nome do perfil.');
  if (!profile.profile_company_name.trim()) issues.push('Informe a empresa vinculada ao perfil.');
  if (!profile.company_municipio_nome.trim()) issues.push('Informe o município da empresa.');
  if (!profile.nfse_layout.trim()) issues.push('Informe o layout municipal da NFS-e.');
  if (!profile.cod_prosoft.trim()) issues.push('Informe o código Prosoft.');
  if (!profile.especie_documento.trim()) issues.push('Informe a espécie do documento.');
  return issues;
}
