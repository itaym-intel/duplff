<script lang="ts">
  import type { DuplicateGroup, RankedFile } from '$lib/types';
  import { formatBytes, truncatePath, confidenceLevel, fileName } from '$lib/format';
  import { markedFiles, keepOverrides } from '$lib/stores';
  import RecommendationBadge from './RecommendationBadge.svelte';
  import FileRow from './FileRow.svelte';
  import FilePreview from './FilePreview.svelte';

  let {
    group,
    index,
    expanded = false,
    focused = false,
    ignored = false,
    onToggleExpand,
    onIgnore,
    onUnignore,
  }: {
    group: DuplicateGroup;
    index: number;
    expanded?: boolean;
    focused?: boolean;
    ignored?: boolean;
    onToggleExpand: () => void;
    onIgnore?: () => void;
    onUnignore?: () => void;
  } = $props();

  let previewPath = $state<string | null>(null);
  let keepDropHover = $state(false);
  let removeDropHover = $state(false);

  // Compute effective keep/duplicates based on user overrides
  let allFiles = $derived([group.keep, ...group.duplicates]);

  let effectiveKeep = $derived.by((): RankedFile => {
    const overridePath = $keepOverrides.get(index);
    if (!overridePath) return group.keep;
    const found = allFiles.find(f => f.entry.path === overridePath);
    return found ?? group.keep;
  });

  let effectiveDuplicates = $derived.by((): RankedFile[] => {
    const keepPath = effectiveKeep.entry.path;
    return allFiles.filter(f => f.entry.path !== keepPath);
  });

  let wasted = $derived(group.size * effectiveDuplicates.length);
  let confidence = $derived(confidenceLevel(group));
  let allDupPaths = $derived(effectiveDuplicates.map(d => d.entry.path));
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

  // Drag and drop handlers
  function handleDragStart(e: DragEvent, path: string) {
    if (e.dataTransfer) {
      e.dataTransfer.setData('text/plain', path);
      e.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleKeepDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    keepDropHover = true;
  }

  function handleKeepDragLeave() {
    keepDropHover = false;
  }

  function handleKeepDrop(e: DragEvent) {
    e.preventDefault();
    keepDropHover = false;
    const path = e.dataTransfer?.getData('text/plain');
    if (!path || path === effectiveKeep.entry.path) return;

    // Remove the promoted file from markedFiles if it was there
    markedFiles.update(set => {
      const next = new Set(set);
      next.delete(path);
      return next;
    });

    keepOverrides.update(map => {
      const next = new Map(map);
      next.set(index, path);
      return next;
    });
  }

  function handleRemoveDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    removeDropHover = true;
  }

  function handleRemoveDragLeave() {
    removeDropHover = false;
  }

  function handleRemoveDrop(e: DragEvent) {
    e.preventDefault();
    removeDropHover = false;
    const path = e.dataTransfer?.getData('text/plain');
    if (!path || path !== effectiveKeep.entry.path) return;

    // Dropping keep file into remove zone — pick first duplicate as new keep
    const firstDup = effectiveDuplicates[0];
    if (firstDup) {
      markedFiles.update(set => {
        const next = new Set(set);
        next.delete(firstDup.entry.path);
        return next;
      });
      keepOverrides.update(map => {
        const next = new Map(map);
        next.set(index, firstDup.entry.path);
        return next;
      });
    }
  }
</script>

<div
  class="rounded-xl border transition-all duration-200
    {ignored ? 'opacity-40' : ''}
    {focused && !ignored ? 'border-active/40 ring-1 ring-active/15 shadow-lg shadow-active/5' : 'border-border-subtle hover:border-border'}
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
          {fileName(effectiveKeep.entry.path)}
        </span>
        <span class="text-[11px] text-text-muted bg-surface-raised px-2 py-0.5 rounded-md font-mono">
          {effectiveDuplicates.length + 1}
        </span>
        {#if ignored}
          <span class="text-[10px] text-text-muted bg-surface-raised px-2 py-0.5 rounded-md uppercase tracking-wider">Ignored</span>
        {/if}
      </div>
      {#if !expanded}
        <p class="font-mono text-xs text-text-muted truncate mt-1">
          {truncatePath(effectiveKeep.entry.path, 80)}
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
      <!-- Keep file (drop zone) -->
      <div class="pt-4"
        ondragover={handleKeepDragOver}
        ondragleave={handleKeepDragLeave}
        ondrop={handleKeepDrop}
        role="region"
      >
        <div class="flex items-center gap-2 mb-2">
          <div class="w-1.5 h-1.5 rounded-full bg-keep"></div>
          <span class="text-[11px] font-medium text-keep uppercase tracking-widest">Keep</span>
          <span class="text-[10px] text-text-muted ml-1">— drag here to keep</span>
        </div>
        <div class="rounded-lg transition-colors {keepDropHover ? 'bg-keep/10 ring-1 ring-keep/30' : ''}">
          <FileRow
            file={effectiveKeep}
            isKeep={true}
            draggable={true}
            onDragStart={(e) => handleDragStart(e, effectiveKeep.entry.path)}
            onPreview={() => togglePreview(effectiveKeep.entry.path)}
          />
        </div>
        <div class="ml-8 mt-2">
          <RecommendationBadge reason={effectiveKeep.reason} {confidence} />
        </div>
        {#if previewPath === effectiveKeep.entry.path}
          <div class="ml-8 mt-2">
            <FilePreview path={effectiveKeep.entry.path} />
          </div>
        {/if}
      </div>

      <!-- Duplicates (drop zone) -->
      <div
        ondragover={handleRemoveDragOver}
        ondragleave={handleRemoveDragLeave}
        ondrop={handleRemoveDrop}
        role="region"
      >
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <div class="w-1.5 h-1.5 rounded-full bg-delete/60"></div>
            <span class="text-[11px] font-medium text-text-muted uppercase tracking-widest">
              {effectiveDuplicates.length === 1 ? '1 copy' : `${effectiveDuplicates.length} copies`}
            </span>
            <span class="text-[10px] text-text-muted ml-1">— drag here to remove</span>
          </div>
          <button
            onclick={selectAllDuplicates}
            class="text-[11px] text-text-muted hover:text-text-secondary transition-colors uppercase tracking-wider"
          >
            Select all
          </button>
        </div>
        <div class="space-y-0.5 rounded-lg transition-colors {removeDropHover ? 'bg-delete/5 ring-1 ring-delete/20' : ''}">
          {#each effectiveDuplicates as dup}
            <FileRow
              file={dup}
              isMarked={$markedFiles.has(dup.entry.path)}
              draggable={true}
              onDragStart={(e) => handleDragStart(e, dup.entry.path)}
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
        {#if ignored && onUnignore}
          <button
            onclick={onUnignore}
            class="text-[11px] text-active/80 hover:text-active bg-active/6 hover:bg-active/10
              px-3 py-1.5 rounded-lg transition-colors font-medium uppercase tracking-wider"
          >
            Unignore
          </button>
        {:else if onIgnore}
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
