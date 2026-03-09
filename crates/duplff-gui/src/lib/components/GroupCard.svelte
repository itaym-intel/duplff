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
    {focused ? 'border-active/50 ring-1 ring-active/20' : 'border-border-subtle'}
    {expanded ? 'bg-surface' : 'bg-surface/50 hover:bg-surface'}"
  role="button"
  tabindex="-1"
>
  <!-- Collapsed Header — always visible -->
  <button
    class="w-full flex items-center gap-4 px-4 py-3 text-left"
    onclick={onToggleExpand}
  >
    <span class="text-text-muted text-xs transition-transform {expanded ? 'rotate-90' : ''}">&#9654;</span>

    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <span class="font-mono text-sm text-text-primary truncate">
          {fileName(group.keep.entry.path)}
        </span>
        <span class="text-xs text-text-muted">
          {group.duplicates.length + 1} files
        </span>
      </div>
      {#if !expanded}
        <p class="font-mono text-xs text-text-muted truncate mt-0.5">
          {truncatePath(group.keep.entry.path, 80)}
        </p>
      {/if}
    </div>

    <div class="flex items-center gap-4 shrink-0">
      <span class="font-mono text-xs text-text-secondary">{formatBytes(group.size)}</span>
      <span class="font-mono text-xs text-delete">{formatBytes(wasted)} wasted</span>
    </div>
  </button>

  <!-- Expanded Content -->
  {#if expanded}
    <div class="px-4 pb-4 space-y-3 border-t border-border-subtle">
      <!-- Keep file -->
      <div class="pt-3">
        <div class="flex items-center gap-2 mb-1.5">
          <span class="text-xs font-medium text-keep uppercase tracking-wide">Recommended</span>
        </div>
        <FileRow
          file={group.keep}
          isKeep={true}
          onPreview={() => togglePreview(group.keep.entry.path)}
        />
        <div class="ml-7 mt-1.5">
          <RecommendationBadge reason={group.keep.reason} {confidence} />
        </div>
        {#if previewPath === group.keep.entry.path}
          <div class="ml-7">
            <FilePreview path={group.keep.entry.path} />
          </div>
        {/if}
      </div>

      <!-- Duplicates -->
      <div>
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-xs font-medium text-text-muted uppercase tracking-wide">
            {group.duplicates.length === 1 ? '1 copy' : `${group.duplicates.length} copies`}
          </span>
          <button
            onclick={selectAllDuplicates}
            class="text-xs text-text-muted hover:text-text-secondary transition-colors"
          >
            Select all
          </button>
        </div>
        {#each group.duplicates as dup}
          <FileRow
            file={dup}
            isMarked={$markedFiles.has(dup.entry.path)}
            onToggle={() => toggleFile(dup.entry.path)}
            onPreview={() => togglePreview(dup.entry.path)}
          />
          {#if previewPath === dup.entry.path}
            <div class="ml-7">
              <FilePreview path={dup.entry.path} />
            </div>
          {/if}
        {/each}
      </div>

      <!-- Per-group actions -->
      <div class="flex items-center gap-2 pt-2 border-t border-border-subtle">
        {#if markedCount > 0}
          <span class="text-xs text-text-muted">{markedCount} selected</span>
        {:else}
          <button
            onclick={selectAllDuplicates}
            class="text-xs text-delete/80 hover:text-delete bg-delete/5 hover:bg-delete/10 px-3 py-1.5 rounded-lg transition-colors"
          >
            Delete others
          </button>
        {/if}
        {#if onIgnore}
          <button
            onclick={onIgnore}
            class="text-xs text-text-muted hover:text-text-secondary px-3 py-1.5 rounded-lg transition-colors"
          >
            Ignore group
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
