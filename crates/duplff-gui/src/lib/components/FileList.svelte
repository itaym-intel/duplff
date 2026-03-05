<script lang="ts">
  import type { RankedFile } from '$lib/types';
  import { reasonLabel } from '$lib/format';
  import { openInFileManager } from '$lib/api';

  export let keep: RankedFile;
  export let duplicates: RankedFile[];
  export let markedPaths: Set<string>;
  export let onToggle: (path: string) => void;
</script>

<div class="space-y-0.5">
  <!-- Keep file -->
  <div class="flex items-center gap-3 rounded-md px-3 py-2 border-l-2 border-keep group">
    <div class="flex-1 min-w-0">
      <p class="font-mono text-xs text-gray-300 truncate" title={keep.entry.path}>{keep.entry.path}</p>
    </div>
    <span class="text-[10px] text-gray-600">{reasonLabel(keep.reason)}</span>
    <button on:click={() => openInFileManager(keep.entry.path)}
      class="text-[10px] text-gray-700 hover:text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
      aria-label="Open in file manager">&#x1F4C2;</button>
  </div>

  <!-- Duplicates -->
  {#each duplicates as dup}
    <div class="flex items-center gap-3 rounded-md px-3 py-2 border-l-2 border-transparent hover:bg-gray-800/30 group transition-colors">
      <input type="checkbox" checked={markedPaths.has(dup.entry.path)}
        on:change={() => onToggle(dup.entry.path)}
        class="w-3.5 h-3.5 rounded accent-active shrink-0" />
      <div class="flex-1 min-w-0">
        <p class="font-mono text-xs text-gray-400 truncate" title={dup.entry.path}>{dup.entry.path}</p>
      </div>
      <button on:click={() => openInFileManager(dup.entry.path)}
        class="text-[10px] text-gray-700 hover:text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
        aria-label="Open in file manager">&#x1F4C2;</button>
    </div>
  {/each}
</div>
