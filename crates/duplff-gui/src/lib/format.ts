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
