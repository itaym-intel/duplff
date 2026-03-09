<script lang="ts">
  import type { DuplicateGroup } from '$lib/types';
  import { formatBytes, truncatePath, confidenceLevel, fileName } from '$lib/format';
  import { markedFiles } from '$lib/stores';
  import RecommendationBadge from './RecommendationBadge.svelte';
  import FileRow from './FileRow.svelte';
  import FilePreview from './FilePreview.svelte';

  let {
    group,
    index,
    expanded = false,
    focused = false,
    onToggleExpand,
    onIgnore,
  }: {
    group: DuplicateGroup;
    index: number;
    expanded?: boolean;
    focused?: boolean;
    onToggleExpand: () => void;
    onIgnore?: () => void;
  } = $props();

  let previewPath = $state<string | null>(null);
  let wasted = $derived(group.size * group.duplicates.length);
  let confidence = $derived(confidenceLevel(group));
  let allDupPaths = $derived(group.duplicates.map(d => d.entry.path));
  let markedCount = $derived(
    allDupPaths.filter(p => $markedFiles.has(p)).length
  );

  function toggleFile(path: string) {
    markedFiles.update(set => {
      const next = new Set(set);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  function selectAllDuplicates() {
    markedFiles.update(set => {
      const next = new Set(set);
      for (const p of allDupPaths) next.add(p);
      return next;
    });
  }

  function togglePreview(path: string) {
    previewPath = previewPath === path ? null : path;
  }
</script>

<div
  class="rounded-xl border transition-all duration-200
    {focused ? 'border-active/40 ring-1 ring-active/15 shadow-lg shadow-active/5' : 'border-border-subtle hover:border-border'}
    {expanded ? 'bg-surface' : 'bg-surface/40 hover:bg-surface/70'}"
  role="button"
  tabindex="-1"
>
  <!-- Collapsed Header -->
  <button
    class="w-full flex items-center gap-4 px-4 py-3.5 text-left"
    onclick={onToggleExpand}
  >
    <span class="text-text-muted text-[10px] transition-transform duration-150 {expanded ? 'rotate-90' : ''}">&#9654;</span>

    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2.5">
        <span class="font-mono text-sm text-text-primary truncate">
          {fileName(group.keep.entry.path)}
        </span>
        <span class="text-[11px] text-text-muted bg-surface-raised px-2 py-0.5 rounded-md font-mono">
          {group.duplicates.length + 1}
        </span>
      </div>
      {#if !expanded}
        <p class="font-mono text-xs text-text-muted truncate mt-1">
          {truncatePath(group.keep.entry.path, 80)}
        </p>
      {/if}
    </div>

    <div class="flex items-center gap-4 shrink-0">
      <span class="font-mono text-xs text-text-secondary">{formatBytes(group.size)}</span>
      <span class="font-mono text-xs text-delete/80 bg-delete/8 px-2 py-0.5 rounded-md">{formatBytes(wasted)}</span>
    </div>
  </button>

  <!-- Expanded Content -->
  {#if expanded}
    <div class="px-4 pb-4 space-y-4 border-t border-border-subtle animate-[expandIn_150ms_ease-out]">
      <!-- Keep file -->
      <div class="pt-4">
        <div class="flex items-center gap-2 mb-2">
          <div class="w-1.5 h-1.5 rounded-full bg-keep"></div>
          <span class="text-[11px] font-medium text-keep uppercase tracking-widest">Keep</span>
        </div>
        <FileRow
          file={group.keep}
          isKeep={true}
          onPreview={() => togglePreview(group.keep.entry.path)}
        />
        <div class="ml-8 mt-2">
          <RecommendationBadge reason={group.keep.reason} {confidence} />
        </div>
        {#if previewPath === group.keep.entry.path}
          <div class="ml-8 mt-2">
            <FilePreview path={group.keep.entry.path} />
          </div>
        {/if}
      </div>

      <!-- Duplicates -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <div class="w-1.5 h-1.5 rounded-full bg-delete/60"></div>
            <span class="text-[11px] font-medium text-text-muted uppercase tracking-widest">
              {group.duplicates.length === 1 ? '1 copy' : `${group.duplicates.length} copies`}
            </span>
          </div>
          <button
            onclick={selectAllDuplicates}
            class="text-[11px] text-text-muted hover:text-text-secondary transition-colors uppercase tracking-wider"
          >
            Select all
          </button>
        </div>
        <div class="space-y-0.5">
          {#each group.duplicates as dup}
            <FileRow
              file={dup}
              isMarked={$markedFiles.has(dup.entry.path)}
              onToggle={() => toggleFile(dup.entry.path)}
              onPreview={() => togglePreview(dup.entry.path)}
            />
            {#if previewPath === dup.entry.path}
              <div class="ml-8">
                <FilePreview path={dup.entry.path} />
              </div>
            {/if}
          {/each}
        </div>
      </div>

      <!-- Per-group actions -->
      <div class="flex items-center gap-2.5 pt-3 border-t border-border-subtle">
        {#if markedCount > 0}
          <span class="text-xs text-text-muted">
            <span class="font-mono text-text-secondary">{markedCount}</span> selected
          </span>
        {:else}
          <button
            onclick={selectAllDuplicates}
            class="text-[11px] text-delete/80 hover:text-delete bg-delete/6 hover:bg-delete/10
              px-3 py-1.5 rounded-lg transition-colors font-medium uppercase tracking-wider"
          >
            Select copies
          </button>
        {/if}
        {#if onIgnore}
          <button
            onclick={onIgnore}
            class="text-[11px] text-text-muted hover:text-text-secondary px-3 py-1.5 rounded-lg
              hover:bg-surface-hover transition-colors uppercase tracking-wider"
          >
            Ignore
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes expandIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
