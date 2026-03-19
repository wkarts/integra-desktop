import { PageHeader } from '../../../shared/components/PageHeader';
import { useNfseStore } from '../../nfse-servicos/stores/NfseStore';
import { ProfileForm } from '../../nfse-servicos/components/ProfileForm';
import { FieldRuleEditor } from '../../nfse-servicos/components/FieldRuleEditor';
import { saveProfile } from '../../nfse-servicos/services/tauriService';

export default function SettingsPage() {
  const { profile, setProfile, pushLog } = useNfseStore();

  return (
    <div className="stack-lg">
      <PageHeader title="Configurações" subtitle="Perfil persistido no app data do Tauri." actions={<button className="btn primary" onClick={async () => { await saveProfile(profile); pushLog('Perfil salvo pela tela de configurações.'); }}>Salvar</button>} />
      <ProfileForm value={profile} onChange={setProfile} />
      <FieldRuleEditor value={profile.field_rules} onChange={(rules) => setProfile({ ...profile, field_rules: rules })} />
    </div>
  );
}
