import { RouterProvider } from 'react-router-dom';
import { AppRouter } from './router';
import { NfseStoreProvider } from '../modules/nfse-servicos/stores/NfseStore';
import { StartupRegistrationGate } from '../modules/licensing/components/StartupRegistrationGate';

export default function App() {
  return (
    <NfseStoreProvider>
      <StartupRegistrationGate />
      <RouterProvider router={AppRouter} />
    </NfseStoreProvider>
  );
}
