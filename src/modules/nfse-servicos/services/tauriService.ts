import { invoke } from '@tauri-apps/api/core';
import type { ConversionProfile, NfseDocument, ProcessBatchInputItem, ProcessBatchResult } from '../../../shared/types';

export async function processNfseBatch(items: ProcessBatchInputItem[]): Promise<ProcessBatchResult> {
  return invoke<ProcessBatchResult>('process_nfse_xml_batch', { items });
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

export async function appendLog(message: string): Promise<void> {
  return invoke('append_runtime_log', { message });
}

export async function listLogs(): Promise<string[]> {
  return invoke<string[]>('list_runtime_logs');
}
