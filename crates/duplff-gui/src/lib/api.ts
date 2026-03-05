import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ScanConfig, DuplicateReport, TrashResult, UndoResult, HashProgress,
  DryRunPlan, ActionLogSummary
} from './types';

export async function startScan(config: ScanConfig): Promise<void> {
  return invoke('start_scan', { config });
}

export async function getResults(): Promise<DuplicateReport | null> {
  return invoke('get_results');
}

export async function trashFiles(paths: string[]): Promise<TrashResult> {
  return invoke('trash_files', { paths });
}

export async function undoLast(): Promise<UndoResult> {
  return invoke('undo_last');
}

export async function exportJson(): Promise<string> {
  return invoke('export_json');
}

export async function exportCsv(): Promise<string> {
  return invoke('export_csv');
}

export async function openInFileManager(path: string): Promise<void> {
  return invoke('open_in_file_manager', { path });
}

export async function dryRun(): Promise<DryRunPlan> {
  return invoke('dry_run');
}

export async function listActionLogs(): Promise<ActionLogSummary[]> {
  return invoke('list_action_logs');
}

export async function undoLog(timestamp: string): Promise<UndoResult> {
  return invoke('undo_log', { timestamp });
}

export function onScanProgress(cb: (filesFound: number) => void): Promise<UnlistenFn> {
  return listen<number>('scan-progress', (e) => cb(e.payload));
}

export function onHashProgress(cb: (progress: HashProgress) => void): Promise<UnlistenFn> {
  return listen<HashProgress>('hash-progress', (e) => cb(e.payload));
}

export function onScanComplete(cb: (report: DuplicateReport) => void): Promise<UnlistenFn> {
  return listen<DuplicateReport>('scan-complete', (e) => cb(e.payload));
}

export function onScanError(cb: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>('scan-error', (e) => cb(e.payload));
}
