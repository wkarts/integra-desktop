import { invoke } from '@tauri-apps/api/core';
import type {
  AppMeta,
  ConversionProfile,
  LicenseRuntimeStatus,
  LicenseSettings,
  NfseDocument,
  ProcessBatchInputItem,
  ProcessBatchResult,
  ProfileBundle,
  UploadInputItem,
} from '../../../shared/types';

export async function processNfseBatch(items: ProcessBatchInputItem[]): Promise<ProcessBatchResult> {
  return invoke<ProcessBatchResult>('process_nfse_xml_batch', { items, profile: null });
}

export async function processNfseUploadBatch(items: UploadInputItem[], profile: ConversionProfile): Promise<ProcessBatchResult> {
  return invoke<ProcessBatchResult>('process_nfse_upload_batch', { items, profile });
}

export async function exportTxt(documents: NfseDocument[], profile: ConversionProfile): Promise<string> {
  return invoke<string>('export_nfse_txt', { documents, profile });
}

export async function exportCsv(documents: NfseDocument[], profile: ConversionProfile): Promise<string> {
  return invoke<string>('export_nfse_csv', { documents, profile });
}

export async function saveProfile(profile: ConversionProfile): Promise<void> {
  return invoke('save_conversion_profile', { profile });
}

export async function loadProfile(): Promise<ConversionProfile | null> {
  return invoke<ConversionProfile | null>('load_conversion_profile');
}

export async function saveProfileBundle(bundle: ProfileBundle): Promise<void> {
  return invoke('save_profile_bundle', { bundle });
}

export async function loadProfileBundle(): Promise<ProfileBundle | null> {
  return invoke<ProfileBundle | null>('load_profile_bundle');
}

export async function appendLog(message: string): Promise<void> {
  return invoke('append_runtime_log', { message });
}

export async function listLogs(): Promise<string[]> {
  return invoke<string[]>('list_runtime_logs');
}

export async function loadLicenseSettings(): Promise<LicenseSettings | null> {
  return invoke<LicenseSettings | null>('load_license_settings');
}

export async function saveLicenseSettings(settings: LicenseSettings): Promise<LicenseSettings> {
  return invoke<LicenseSettings>('save_license_settings', { settings });
}

export async function checkLicenseStatus(settings: LicenseSettings): Promise<LicenseRuntimeStatus> {
  return invoke<LicenseRuntimeStatus>('check_license_status', { settings });
}

export async function getMachineFingerprint(): Promise<string> {
  return invoke<string>('get_machine_fingerprint');
}

export async function getDefaultStationName(): Promise<string> {
  return invoke<string>('get_default_station_name');
}

export async function getAppMeta(): Promise<AppMeta> {
  return invoke<AppMeta>('get_app_meta');
}
