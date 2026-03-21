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
  const dateLabel = now.toLocaleDateString('pt-BR', {
    weekday: 'long',
    day: '2-digit',
    month: 'short',
  });

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <img src={brandLogo} alt="Integra Web" className="sidebar-logo" />
          <div>
            <h1>Integra Desktop</h1>
            <p className="muted sidebar-subtitle">Operação fiscal e licenciamento.</p>
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
          <section className="sidebar-clock" aria-label="Relógio atual">
            <div className="clock-face" role="img" aria-label={`Horário atual: ${timeLabel}`}>
              <span className="clock-hand clock-hand-hour" style={{ transform: `translateX(-50%) rotate(${hourAngle}deg)` }} />
              <span className="clock-hand clock-hand-minute" style={{ transform: `translateX(-50%) rotate(${minuteAngle}deg)` }} />
              <span className="clock-hand clock-hand-second" style={{ transform: `translateX(-50%) rotate(${secondAngle}deg)` }} />
              <span className="clock-center" />
            </div>
            <strong className="clock-time">{timeLabel}</strong>
            <span className="clock-date">{dateLabel}</span>
          </section>
          <div className="sidebar-meta">
            <div className="sidebar-meta-row"><span>Versão</span><strong>{meta.version}</strong></div>
            <div className="sidebar-meta-row"><span>ASHA</span><strong title={meta.build_hash}>{meta.build_hash.slice(0, 12)}</strong></div>
          </div>
        </footer>
      </aside>
      <main className="main-content">{children}</main>
    </div>
  );
}
