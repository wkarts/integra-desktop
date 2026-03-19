import { createContext, useContext, useMemo, useState, type PropsWithChildren } from 'react';
import type { ConversionProfile, NfseDocument } from '../../../shared/types';
import { defaultProfile } from '../../../shared/mappers/defaultProfile';

interface NfseStoreState {
  documents: NfseDocument[];
  profile: ConversionProfile;
  logs: string[];
  setDocuments: (documents: NfseDocument[]) => void;
  updateDocument: (id: string, patch: Partial<NfseDocument>) => void;
  setProfile: (profile: ConversionProfile) => void;
  pushLog: (message: string) => void;
  clearAll: () => void;
}

const NfseStoreContext = createContext<NfseStoreState | null>(null);

export function NfseStoreProvider({ children }: PropsWithChildren) {
  const [documents, setDocuments] = useState<NfseDocument[]>([]);
  const [profile, setProfile] = useState<ConversionProfile>(defaultProfile);
  const [logs, setLogs] = useState<string[]>([]);

  const value = useMemo<NfseStoreState>(() => ({
    documents,
    profile,
    logs,
    setDocuments,
    updateDocument: (id, patch) => {
      setDocuments((current) => current.map((item) => item.id === id ? { ...item, ...patch } : item));
    },
    setProfile,
    pushLog: (message) => setLogs((current) => [...current, `[${new Date().toLocaleTimeString('pt-BR')}] ${message}`]),
    clearAll: () => {
      setDocuments([]);
      setLogs([]);
      setProfile(defaultProfile);
    },
  }), [documents, logs, profile]);

  return <NfseStoreContext.Provider value={value}>{children}</NfseStoreContext.Provider>;
}

export function useNfseStore() {
  const ctx = useContext(NfseStoreContext);
  if (!ctx) {
    throw new Error('useNfseStore deve ser usado dentro de NfseStoreProvider');
  }
  return ctx;
}
