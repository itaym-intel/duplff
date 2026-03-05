import type { Confidence, DuplicateGroup } from './types';

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

export function truncatePath(path: string, maxLen = 60): string {
  if (path.length <= maxLen) return path;
  return '...' + path.slice(-(maxLen - 3));
}

export function reasonLabel(reason: string): string {
  const labels: Record<string, string> = {
    PriorityPath: 'Priority directory',
    DeepestPath: 'Deepest path',
    NewestModification: 'Newest file',
    LexicographicFirst: 'First alphabetically',
  };
  return labels[reason] || reason;
}

export function confidenceLevel(group: DuplicateGroup): Confidence {
  const reason = group.keep.reason;
  if (reason === 'PriorityPath') return 'high';
  if (reason === 'NewestModification') return 'high';
  if (reason === 'DeepestPath') return 'medium';
  return 'low';
}

export function confidenceColor(level: Confidence): string {
  switch (level) {
    case 'high': return 'text-keep';
    case 'medium': return 'text-warn';
    case 'low': return 'text-text-muted';
  }
}

export function fileExtension(path: string): string {
  const dot = path.lastIndexOf('.');
  if (dot === -1) return '';
  return path.slice(dot + 1).toLowerCase();
}

export function fileName(path: string): string {
  return path.split('/').pop() || path;
}

export function dirName(path: string): string {
  const parts = path.split('/');
  parts.pop();
  return parts.join('/');
}

export function formatNumber(n: number): string {
  return n.toLocaleString();
}
