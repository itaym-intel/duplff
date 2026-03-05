<script lang="ts">
  import type { RankedFile } from '$lib/types';
  import { openInFileManager } from '$lib/api';
  import { fileName, dirName } from '$lib/format';

  let {
    file,
    isKeep = false,
    isMarked = false,
    onToggle,
    onPreview,
  }: {
    file: RankedFile;
    isKeep?: boolean;
    isMarked?: boolean;
    onToggle?: () => void;
    onPreview?: () => void;
  } = $props();

  let hovered = $state(false);
</script>

<div
  class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors group
    {isKeep ? 'border-l-2 border-keep bg-keep/5' : 'border-l-2 border-transparent hover:bg-surface-hover'}"
  onmouseenter={() => hovered = true}
  onmouseleave={() => hovered = false}
  role="row"
>
  {#if !isKeep && onToggle}
    <input
      type="checkbox"
      checked={isMarked}
      onchange={onToggle}
      class="w-4 h-4 rounded accent-active shrink-0 cursor-pointer"
    />
  {:else}
    <span class="w-4 text-keep text-xs font-bold shrink-0" title="Recommended to keep">*</span>
  {/if}

  <div class="flex-1 min-w-0">
    <div class="flex items-baseline gap-2">
      <span class="font-mono text-sm {isKeep ? 'text-text-primary font-medium' : 'text-text-secondary'} truncate">
        {fileName(file.entry.path)}
      </span>
    </div>
    <p class="font-mono text-xs text-text-muted truncate mt-0.5" title={file.entry.path}>
      {dirName(file.entry.path)}
    </p>
  </div>

  <div class="flex items-center gap-1.5 shrink-0 transition-opacity {hovered ? 'opacity-100' : 'opacity-0'}">
    {#if onPreview}
      <button
        onclick={onPreview}
        class="text-xs text-text-muted hover:text-text-secondary px-1.5 py-0.5 rounded transition-colors"
        title="Preview file"
      >
        Preview
      </button>
    {/if}
    <button
      onclick={() => openInFileManager(file.entry.path)}
      class="text-xs text-text-muted hover:text-text-secondary px-1.5 py-0.5 rounded transition-colors"
      title="Open in file manager"
    >
      Reveal
    </button>
  </div>
</div>
