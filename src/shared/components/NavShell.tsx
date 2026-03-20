import { NavLink } from 'react-router-dom';
import type { PropsWithChildren } from 'react';
import { useEffect, useState } from 'react';
import type { AppMeta } from '../types';
import brandLogo from '../../assets/integra-logo.svg';
import { getAppMeta } from '../../modules/nfse-servicos/services/tauriService';

const menu = [
  { to: '/', label: 'Dashboard' },
  { to: '/nfse-servicos', label: 'NFS-e → Prosoft' },
  { to: '/nfe-faturas', label: 'NFe / Faturas' },
  { to: '/settings', label: 'Configurações' },
  { to: '/logs', label: 'Logs' },
];

const defaultMeta: AppMeta = {
  product_name: 'Integra Desktop',
  version: '1.0.3',
  build_hash: 'dev-local',
  app_id: 'integra-desktop',
};

export function NavShell({ children }: PropsWithChildren) {
  const [meta, setMeta] = useState<AppMeta>(defaultMeta);

  useEffect(() => {
    getAppMeta().then(setMeta).catch(() => setMeta(defaultMeta));
  }, []);

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <img src={brandLogo} alt="Integra Web" className="sidebar-logo" />
          <div>
            <h1>Integra Desktop</h1>
            <p className="muted sidebar-subtitle">Importação fiscal, perfis por empresa e controle de licenças por estação.</p>
          </div>
        </div>

        <nav className="menu">
          {menu.map((item) => (
            <NavLink key={item.to} to={item.to} end={item.to === '/'} className={({ isActive }) => isActive ? 'menu-link active' : 'menu-link'}>
              {item.label}
            </NavLink>
          ))}
        </nav>

        <footer className="sidebar-footer">
          <div className="sidebar-meta-row"><span>Versão</span><strong>{meta.version}</strong></div>
          <div className="sidebar-meta-row"><span>ASHA</span><strong title={meta.build_hash}>{meta.build_hash.slice(0, 12)}</strong></div>
        </footer>
      </aside>
      <main className="main-content">{children}</main>
    </div>
  );
}
