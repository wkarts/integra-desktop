import { createHashRouter, Outlet } from 'react-router-dom';
import DashboardPage from '../modules/dashboard/pages/DashboardPage';
import NfseServicosPage from '../modules/nfse-servicos/pages/NfseServicosPage';
import NfeFaturasPage from '../modules/nfe-faturas/pages/NfeFaturasPage';
import NfseXmlConverterPage from '../modules/nfse-servicos/pages/NfseXmlConverterPage';
import SettingsPage from '../modules/settings/pages/SettingsPage';
import LogsPage from '../modules/logs/pages/LogsPage';
import { NavShell } from '../shared/components/NavShell';

function RootLayout() {
  return (
    <NavShell>
      <Outlet />
    </NavShell>
  );
}

export const AppRouter = createHashRouter([
  {
    path: '/',
    element: <RootLayout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: 'nfse-servicos', element: <NfseServicosPage /> },
      { path: 'nfe-faturas', element: <NfeFaturasPage /> },
      { path: 'nfse-converter', element: <NfseXmlConverterPage /> },
      { path: 'settings', element: <SettingsPage /> },
      { path: 'logs', element: <LogsPage /> },
    ],
  },
]);
