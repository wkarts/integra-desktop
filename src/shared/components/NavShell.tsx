import { NavLink } from 'react-router-dom';
import type { PropsWithChildren } from 'react';

const menu = [
  { to: '/', label: 'Dashboard' },
  { to: '/nfse-servicos', label: 'NFS-e → Prosoft' },
  { to: '/nfe-faturas', label: 'NFe / Faturas' },
  { to: '/legado', label: 'Legado' },
  { to: '/settings', label: 'Configurações' },
  { to: '/logs', label: 'Logs' },
];

export function NavShell({ children }: PropsWithChildren) {
  return (
    <div className="shell">
      <aside className="sidebar">
        <div>
          <h1>Integra Desktop</h1>
          <p className="muted">Tauri + Rust + React/TS com fallback legado HTML.</p>
        </div>
        <nav className="menu">
          {menu.map((item) => (
            <NavLink key={item.to} to={item.to} end={item.to === '/'} className={({ isActive }) => isActive ? 'menu-link active' : 'menu-link'}>
              {item.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="main-content">
        {children}
      </main>
    </div>
  );
}
