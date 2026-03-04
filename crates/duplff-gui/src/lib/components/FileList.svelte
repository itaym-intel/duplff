<script lang="ts">
  import type { RankedFile } from '$lib/types';
  import { formatBytes, reasonLabel } from '$lib/format';
  import { openInFileManager } from '$lib/api';

  export let keep: RankedFile;
  export let duplicates: RankedFile[];
  export let markedPaths: Set<string>;
  export let onToggle: (path: string) => void;
</script>

<div class="space-y-1">
  <div class="flex items-center gap-3 bg-gray-800 rounded-lg px-4 py-3">
    <span class="text-keep text-lg w-5 text-center">✓</span>
    <div class="flex-1 min-w-0">
      <p class="font-mono text-sm truncate" title={keep.entry.path}>{keep.entry.path}</p>
      <p class="text-xs text-gray-500">{formatBytes(keep.entry.size)}</p>
    </div>
    <span class="text-xs bg-keep/20 text-keep px-2 py-0.5 rounded shrink-0">
      Kept: {reasonLabel(keep.reason)}
    </span>
    <button on:click={() => openInFileManager(keep.entry.path)}
      class="text-xs text-gray-500 hover:text-active shrink-0">Open</button>
  </div>

  {#each duplicates as dup}
    <div class="flex items-center gap-3 bg-gray-800/50 rounded-lg px-4 py-3">
      <input type="checkbox" checked={markedPaths.has(dup.entry.path)}
        on:change={() => onToggle(dup.entry.path)} class="w-5 h-5 rounded" />
      <div class="flex-1 min-w-0">
        <p class="font-mono text-sm truncate" title={dup.entry.path}>{dup.entry.path}</p>
        <p class="text-xs text-gray-500">{formatBytes(dup.entry.size)}</p>
      </div>
      <span class="text-xs bg-delete/20 text-delete px-2 py-0.5 rounded shrink-0">Duplicate</span>
      <button on:click={() => openInFileManager(dup.entry.path)}
        class="text-xs text-gray-500 hover:text-active shrink-0">Open</button>
    </div>
  {/each}
</div>
