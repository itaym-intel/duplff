<script lang="ts">
  import { scanConfig, currentScreen } from '$lib/stores';
  import FolderPicker from '$lib/components/FolderPicker.svelte';

  let roots: string[] = $state([]);
  let priorityPaths: string[] = $state([]);
  let extensions = $state('');
  let excludePatterns = $state('');
  let minSize = $state(1);
  let minSizeUnit: 'B' | 'KB' | 'MB' = $state('KB');
  let maxSize = $state('');
  let followSymlinks = $state(false);
  let paranoid = $state(false);
  let noCache = $state(false);
  let showAdvanced = $state(false);

  function getMinSizeBytes(): number {
    const multipliers = { B: 1, KB: 1024, MB: 1024 * 1024 };
    return minSize * multipliers[minSizeUnit];
  }

  function handleScan() {
    if (roots.length === 0) return;
    const config = {
      roots,
      extensions: extensions.trim()
        ? extensions.split(',').map(e => e.trim())
        : null,
      min_size: getMinSizeBytes(),
      max_size: maxSize ? parseInt(maxSize) * 1024 * 1024 : null,
      priority_paths: priorityPaths,
      follow_symlinks: followSymlinks,
      exclude_patterns: excludePatterns.trim()
        ? excludePatterns.split(',').map(p => p.trim())
        : [],
      no_cache: noCache,
      paranoid,
    };
    scanConfig.set(config);
    currentScreen.set('progress');
  }
</script>

<div class="flex items-center justify-center min-h-screen p-8">
  <div class="w-full max-w-md space-y-6">
    <div class="mb-4">
      <h1 class="text-2xl font-semibold tracking-tight">duplff</h1>
      <p class="text-sm text-text-muted mt-1">Find and remove duplicate files</p>
    </div>

    <FolderPicker bind:folders={roots} label="Scan directories" />

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label for="min-size" class="block text-xs font-medium text-text-muted uppercase tracking-wider mb-2">Min size</label>
        <div class="flex">
          <input id="min-size" type="number" bind:value={minSize} min="0"
            class="flex-1 min-w-0 bg-surface border border-border rounded-l-lg px-3 py-2 text-sm focus:border-active/50 focus:outline-none focus:ring-1 focus:ring-active/20" />
          <select bind:value={minSizeUnit} class="bg-surface border border-l-0 border-border rounded-r-lg px-2 text-sm text-text-secondary">
            <option value="B">B</option>
            <option value="KB">KB</option>
            <option value="MB">MB</option>
          </select>
        </div>
      </div>
      <div>
        <label for="max-size" class="block text-xs font-medium text-text-muted uppercase tracking-wider mb-2">Max size</label>
        <div class="flex items-center">
          <input id="max-size" type="number" bind:value={maxSize} placeholder="No limit"
            class="flex-1 min-w-0 bg-surface border border-border rounded-l-lg px-3 py-2 text-sm focus:border-active/50 focus:outline-none focus:ring-1 focus:ring-active/20 placeholder-text-muted" />
          <span class="bg-surface border border-l-0 border-border rounded-r-lg px-2 py-2 text-sm text-text-muted">MB</span>
        </div>
      </div>
    </div>

    <div>
      <label for="extensions" class="block text-xs font-medium text-text-muted uppercase tracking-wider mb-2">Extensions</label>
      <input id="extensions" type="text" bind:value={extensions} placeholder="e.g. py, rs, js"
        class="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm focus:border-active/50 focus:outline-none focus:ring-1 focus:ring-active/20 placeholder-text-muted" />
    </div>

    <button
      onclick={() => showAdvanced = !showAdvanced}
      class="flex items-center gap-2 text-sm text-text-muted hover:text-text-secondary transition-colors"
    >
      <span class="text-xs transition-transform {showAdvanced ? 'rotate-90' : ''}">&#9654;</span>
      Advanced
    </button>

    {#if showAdvanced}
      <div class="space-y-4 pl-3 border-l border-border ml-1">
        <div>
          <label for="exclude" class="block text-xs font-medium text-text-muted uppercase tracking-wider mb-2">Exclude patterns</label>
          <input id="exclude" type="text" bind:value={excludePatterns} placeholder="e.g. node_modules, .git, target"
            class="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm focus:border-active/50 focus:outline-none focus:ring-1 focus:ring-active/20 placeholder-text-muted" />
        </div>

        <FolderPicker bind:folders={priorityPaths} label="Priority directories" />

        <div class="space-y-3">
          <label class="flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer">
            <input type="checkbox" bind:checked={followSymlinks} class="w-4 h-4 rounded accent-active" />
            Follow symlinks
          </label>
          <label class="flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer">
            <input type="checkbox" bind:checked={paranoid} class="w-4 h-4 rounded accent-active" />
            Byte-by-byte verification
          </label>
          <label class="flex items-center gap-2.5 text-sm text-text-secondary cursor-pointer">
            <input type="checkbox" bind:checked={noCache} class="w-4 h-4 rounded accent-active" />
            Disable hash cache
          </label>
        </div>
      </div>
    {/if}

    <button
      onclick={handleScan}
      disabled={roots.length === 0}
      class="w-full bg-active hover:bg-cyan-500 disabled:bg-surface disabled:text-text-muted
        text-white font-medium py-3 rounded-xl transition-colors text-sm"
    >
      Scan for duplicates
    </button>
  </div>
</div>
