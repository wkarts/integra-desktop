import { RouterProvider } from 'react-router-dom';
import { AppRouter } from './router';
import { NfseStoreProvider } from '../modules/nfse-servicos/stores/NfseStore';

export default function App() {
  return (
    <NfseStoreProvider>
      <RouterProvider router={AppRouter} />
    </NfseStoreProvider>
  );
}
