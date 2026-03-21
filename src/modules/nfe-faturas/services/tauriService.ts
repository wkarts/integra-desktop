import { invoke } from '@tauri-apps/api/core';
import type {
  NamedText,
  NfeMeta,
  NfeFaturasExportResult,
  NfeFaturasLegacyResult,
  NfeFaturasProcessResult,
  NfeFaturasRow,
  NfeFaturasSettings,
} from '../types';

export function loadNfeFaturasSettings(): Promise<NfeFaturasSettings> {
  return invoke<NfeFaturasSettings>('load_nfe_faturas_settings');
}

export function saveNfeFaturasSettings(settings: NfeFaturasSettings): Promise<void> {
  return invoke('save_nfe_faturas_settings', { settings });
}

export function processNfeFaturasSelection(paths: string[], settings: NfeFaturasSettings): Promise<NfeFaturasProcessResult> {
  return invoke<NfeFaturasProcessResult>('process_nfe_faturas_selection', { paths, settings });
}

export function importNfeFaturasLegacy(
  filePath: string,
  cnpjFilial: string | null,
  conferir: boolean,
  currentRows: NfeFaturasRow[],
  spedFiles: NamedText[],
  nfeMetas: NfeMeta[],
): Promise<NfeFaturasLegacyResult> {
  return invoke<NfeFaturasLegacyResult>('import_nfe_faturas_legacy', {
    filePath,
    cnpjFilial,
    conferir,
    currentRows,
    spedFiles,
    nfeMetas,
  });
}

export function guessNfeFaturasCnpjFilial(rows: NfeFaturasRow[], spedFiles: NamedText[]): Promise<string | null> {
  return invoke<string | null>('guess_nfe_faturas_cnpj_filial', { rows, spedFiles });
}

export function exportNfeFaturasTxt(
  rows: NfeFaturasRow[],
  settings: NfeFaturasSettings,
  outputPath: string,
): Promise<NfeFaturasExportResult> {
  return invoke<NfeFaturasExportResult>('export_nfe_faturas_txt', { rows, settings, outputPath });
}

export function exportNfeFaturasCsv(
  rows: NfeFaturasRow[],
  settings: NfeFaturasSettings,
  outputPath: string,
): Promise<NfeFaturasExportResult> {
  return invoke<NfeFaturasExportResult>('export_nfe_faturas_csv', { rows, settings, outputPath });
}

export function exportNfeFaturasSped(
  rows: NfeFaturasRow[],
  settings: NfeFaturasSettings,
  spedFiles: NamedText[],
  nfeMetas: NfeMeta[],
  outputDir: string,
): Promise<NfeFaturasExportResult> {
  return invoke<NfeFaturasExportResult>('export_nfe_faturas_sped', {
    rows,
    settings,
    spedFiles,
    nfeMetas,
    outputDir,
  });
}


export function dialogPickNfeFaturasFiles(): Promise<string[]> {
  return invoke<string[]>('dialog_pick_nfe_faturas_files');
}

export function dialogPickNfeFaturasDirectory(): Promise<string | null> {
  return invoke<string | null>('dialog_pick_nfe_faturas_directory');
}

export function dialogPickNfeFaturasLegacyFile(): Promise<string | null> {
  return invoke<string | null>('dialog_pick_nfe_faturas_legacy_file');
}

export function dialogPickNfeFaturasOutputDir(): Promise<string | null> {
  return invoke<string | null>('dialog_pick_nfe_faturas_output_dir');
}

export function dialogSaveNfeFaturasFile(defaultName: string, title: string, extensions: string[]): Promise<string | null> {
  return invoke<string | null>('dialog_save_nfe_faturas_file', { defaultName, title, extensions });
}

export function dialogMessageInfo(title: string, message: string): Promise<void> {
  return invoke('dialog_message_info', { title, message });
}

export function dialogMessageWarning(title: string, message: string): Promise<void> {
  return invoke('dialog_message_warning', { title, message });
}

export function dialogMessageError(title: string, message: string): Promise<void> {
  return invoke('dialog_message_error', { title, message });
}

export function dialogConfirm(title: string, message: string): Promise<boolean> {
  return invoke<boolean>('dialog_confirm', { title, message });
}

export function clipboardWriteText(text: string): Promise<void> {
  return invoke('clipboard_write_text', { text });
}
