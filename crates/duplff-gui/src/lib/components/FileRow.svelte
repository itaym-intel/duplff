<script lang="ts">
  import type { RankedFile } from '$lib/types';
  import { openInFileManager } from '$lib/api';
  import { fileName, dirName } from '$lib/format';

  let {
    file,
    isKeep = false,
    isMarked = false,
    draggable = false,
    onToggle,
    onPreview,
    onDragStart,
  }: {
    file: RankedFile;
    isKeep?: boolean;
    isMarked?: boolean;
    draggable?: boolean;
    onToggle?: () => void;
    onPreview?: () => void;
    onDragStart?: (e: DragEvent) => void;
  } = $props();

  let hovered = $state(false);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all group
    {draggable ? 'cursor-grab active:cursor-grabbing' : ''}
    {isKeep
      ? 'border-l-2 border-keep/60 bg-keep/5 hover:bg-keep/8'
      : isMarked
        ? 'border-l-2 border-delete/40 bg-delete/5 hover:bg-delete/8'
        : 'border-l-2 border-transparent hover:bg-surface-hover'}"
  draggable={draggable ? 'true' : undefined}
  ondragstart={onDragStart}
  onmouseenter={() => hovered = true}
  onmouseleave={() => hovered = false}
>
  {#if !isKeep && onToggle}
    <input
      type="checkbox"
      checked={isMarked}
      onchange={onToggle}
      class="w-4 h-4 rounded accent-active shrink-0 cursor-pointer"
    />
  {:else}
    <div class="w-4 h-4 rounded-full bg-keep/20 flex items-center justify-center shrink-0">
      <div class="w-1.5 h-1.5 rounded-full bg-keep"></div>
    </div>
  {/if}

  <div class="flex-1 min-w-0">
    <span class="font-mono text-sm {isKeep ? 'text-text-primary' : 'text-text-secondary'} truncate block">
      {fileName(file.entry.path)}
    </span>
    <p class="font-mono text-[11px] text-text-muted truncate mt-0.5" title={file.entry.path}>
      {dirName(file.entry.path)}
    </p>
  </div>

  {#if draggable}
    <span class="text-text-muted/40 text-xs shrink-0 select-none" title="Drag to reorder">⠿</span>
  {/if}

  <div class="flex items-center gap-1 shrink-0 transition-opacity duration-150 {hovered ? 'opacity-100' : 'opacity-0'}">
    {#if onPreview}
      <button
        onclick={onPreview}
        class="text-[11px] text-text-muted hover:text-active px-2 py-1 rounded-md
          hover:bg-active/10 transition-colors font-medium"
        title="Preview file"
      >
        Preview
      </button>
    {/if}
    <button
      onclick={() => openInFileManager(file.entry.path)}
      class="text-[11px] text-text-muted hover:text-text-secondary px-2 py-1 rounded-md
        hover:bg-surface-hover transition-colors font-medium"
      title="Open in file manager"
    >
      Reveal
    </button>
  </div>
</div>
