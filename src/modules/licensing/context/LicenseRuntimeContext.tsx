import { createContext, useContext, useMemo, useState, type PropsWithChildren } from 'react';
import type { LicenseRuntimeStatus, StartupLicenseContext } from '../../../shared/types';

interface LicenseRuntimeContextValue {
  status: LicenseRuntimeStatus | null;
  setStatus: (status: LicenseRuntimeStatus | null) => void;
  startupContext: StartupLicenseContext | null;
  setStartupContext: (context: StartupLicenseContext | null) => void;
}

const LicenseRuntimeContext = createContext<LicenseRuntimeContextValue | undefined>(undefined);

export function LicenseRuntimeProvider({ children }: PropsWithChildren) {
  const [status, setStatus] = useState<LicenseRuntimeStatus | null>(null);
  const [startupContext, setStartupContext] = useState<StartupLicenseContext | null>(null);

  const value = useMemo(
    () => ({
      status,
      setStatus,
      startupContext,
      setStartupContext,
    }),
    [status, startupContext],
  );

  return (
    <LicenseRuntimeContext.Provider value={value}>
      {children}
    </LicenseRuntimeContext.Provider>
  );
}

export function useLicenseRuntime() {
  const context = useContext(LicenseRuntimeContext);
  if (!context) {
    throw new Error('useLicenseRuntime deve ser usado dentro de LicenseRuntimeProvider');
  }
  return context;
}
