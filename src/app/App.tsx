import { RouterProvider } from 'react-router-dom';
import { AppRouter } from './router';
import { NfseStoreProvider } from '../modules/nfse-servicos/stores/NfseStore';
import { StartupRegistrationGate } from '../modules/licensing/components/StartupRegistrationGate';
import { LicenseRuntimeProvider } from '../modules/licensing/context/LicenseRuntimeContext';

export default function App() {
  return (
    <NfseStoreProvider>
      <LicenseRuntimeProvider>
        <StartupRegistrationGate />
        <RouterProvider router={AppRouter} />
      </LicenseRuntimeProvider>
    </NfseStoreProvider>
  );
}
