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
  const [menuOpen, setMenuOpen] = useState(false);
  const [meta, setMeta] = useState<AppMeta>(defaultMeta);
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    getAppMeta().then(setMeta).catch(() => setMeta(defaultMeta));
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const hour = now.getHours() % 12;
  const minute = now.getMinutes();
  const second = now.getSeconds();
  const hourAngle = hour * 30 + minute * 0.5;
  const minuteAngle = minute * 6 + second * 0.1;
  const secondAngle = second * 6;
  const timeLabel = now.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' });
  const dateLabel = now.toLocaleDateString('pt-BR', { weekday: 'long', day: '2-digit', month: 'short' });

  return (
    <div className="shell">
      <aside className={`sidebar ${menuOpen ? 'open' : ''}`}>
        <div className="sidebar-brand">
          <div className="sidebar-brand-top">
            <img src={brandLogo} alt="Integra Web" className="sidebar-logo" />
            <button
              type="button"
              className="sidebar-toggle"
              onClick={() => setMenuOpen((prev) => !prev)}
              aria-label={menuOpen ? 'Fechar menu' : 'Abrir menu'}
              aria-expanded={menuOpen}
            >
              <span />
              <span />
              <span />
            </button>
          </div>
          <div className="sidebar-brand-text">
            <h1>Integra Desktop</h1>
            <p className="muted sidebar-subtitle">Operação fiscal e licenciamento.</p>
          </div>
        </div>

        <nav className="menu">
          {menu.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/'}
              className={({ isActive }) => isActive ? 'menu-link active' : 'menu-link'}
              onClick={() => setMenuOpen(false)}
            >
              {item.label}
            </NavLink>
          ))}
        </nav>

        <footer className="sidebar-footer">
          <section className="sidebar-system-panel" aria-label="Relógio e versão do sistema">
            <div className="dashboard-clock" role="img" aria-label={`Horário atual: ${timeLabel}`}>
              <span className="clock-hand clock-hand-hour" style={{ transform: `translateX(-50%) rotate(${hourAngle}deg)` }} />
              <span className="clock-hand clock-hand-minute" style={{ transform: `translateX(-50%) rotate(${minuteAngle}deg)` }} />
              <span className="clock-hand clock-hand-second" style={{ transform: `translateX(-50%) rotate(${secondAngle}deg)` }} />
              <span className="clock-center" />
            </div>
            <div className="dashboard-system-meta">
              <strong className="clock-time">{timeLabel}</strong>
              <span className="clock-date">{dateLabel}</span>
              <div className="dashboard-version">
                <span>Versão {meta.version}</span>
                <span>ASHA {meta.build_hash.slice(0, 12)}</span>
              </div>
            </div>
          </section>
        </footer>
      </aside>
      <main className="main-content">{children}</main>
    </div>
  );
}
