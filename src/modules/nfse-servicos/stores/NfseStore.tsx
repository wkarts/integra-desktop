import { createContext, useCallback, useContext, useMemo, useState, type PropsWithChildren } from 'react';
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

  const updateDocument = useCallback((id: string, patch: Partial<NfseDocument>) => {
    setDocuments((current) => current.map((item) => item.id === id ? { ...item, ...patch } : item));
  }, []);

  const pushLog = useCallback((message: string) => {
    setLogs((current) => [...current, `[${new Date().toLocaleTimeString('pt-BR')}] ${message}`]);
  }, []);

  const clearAll = useCallback(() => {
    setDocuments([]);
    setLogs([]);
    setProfile(defaultProfile);
  }, []);

  const value = useMemo<NfseStoreState>(() => ({
    documents,
    profile,
    logs,
    setDocuments,
    updateDocument,
    setProfile,
    pushLog,
    clearAll,
  }), [documents, logs, profile, updateDocument, pushLog, clearAll]);

  return <NfseStoreContext.Provider value={value}>{children}</NfseStoreContext.Provider>;
}

export function useNfseStore() {
  const ctx = useContext(NfseStoreContext);
  if (!ctx) {
    throw new Error('useNfseStore deve ser usado dentro de NfseStoreProvider');
  }
  return ctx;
}
