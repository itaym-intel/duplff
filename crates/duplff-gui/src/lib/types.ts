export interface ScanConfig {
  roots: string[];
  extensions: string[] | null;
  min_size: number;
  max_size: number | null;
  priority_paths: string[];
  follow_symlinks: boolean;
  exclude_patterns: string[];
  no_cache: boolean;
  paranoid: boolean;
}

export interface FileEntry {
  path: string;
  size: number;
  modified: number;
}

export interface RankedFile {
  entry: FileEntry;
  reason: 'PriorityPath' | 'DeepestPath' | 'NewestModification' | 'LexicographicFirst';
}

export interface DuplicateGroup {
  hash: string;
  size: number;
  keep: RankedFile;
  duplicates: RankedFile[];
}

export interface DuplicateReport {
  groups: DuplicateGroup[];
  total_files_scanned: number;
  total_bytes_scanned: number;
  total_duplicates: number;
  total_wasted_bytes: number;
}

export interface TrashResult {
  count: number;
  bytes_reclaimed: number;
}

export interface UndoResult {
  restored: number;
  not_found: number;
}

export interface HashProgress {
  done: number;
  total: number;
}

export interface DryRunPlan {
  files_to_delete: string[];
  bytes_to_reclaim: number;
  group_count: number;
}

export interface ActionLogSummary {
  timestamp: string;
  file_count: number;
  bytes_reclaimed: number;
}

export type Screen = 'setup' | 'progress' | 'results' | 'detail';
export type SortMode = 'wasted' | 'size' | 'files' | 'path';

export function defaultConfig(): ScanConfig {
  return {
    roots: [],
    extensions: null,
    min_size: 1024,
    max_size: null,
    priority_paths: [],
    follow_symlinks: false,
    exclude_patterns: [],
    no_cache: false,
    paranoid: false,
  };
}
