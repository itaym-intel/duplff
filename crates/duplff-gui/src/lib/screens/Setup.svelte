<script lang="ts">
  import { scanConfig, currentScreen } from '$lib/stores';
  import { startScan } from '$lib/api';
  import FolderPicker from '$lib/components/FolderPicker.svelte';

  let roots: string[] = [];
  let priorityPaths: string[] = [];
  let extensions = '';
  let excludePatterns = '';
  let minSize = 1;
  let minSizeUnit: 'B' | 'KB' | 'MB' = 'KB';
  let maxSize = '';
  let followSymlinks = false;
  let paranoid = false;
  let noCache = false;
  let showAdvanced = false;

  function getMinSizeBytes(): number {
    const multipliers = { B: 1, KB: 1024, MB: 1024 * 1024 };
    return minSize * multipliers[minSizeUnit];
  }

  async function handleScan() {
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
    await startScan(config);
  }
</script>

<div class="flex items-center justify-center min-h-screen p-6">
  <div class="w-full max-w-md space-y-6">
    <div class="mb-2">
      <h1 class="text-xl font-semibold tracking-tight">duplff</h1>
      <p class="text-sm text-gray-500 mt-0.5">Find and remove duplicate files</p>
    </div>

    <FolderPicker bind:folders={roots} label="Scan directories" />

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label for="min-size" class="block text-xs font-medium text-gray-500 uppercase tracking-wide mb-1.5">Min size</label>
        <div class="flex">
          <input id="min-size" type="number" bind:value={minSize} min="0"
            class="flex-1 min-w-0 bg-gray-800 border border-gray-700 rounded-l-md px-2.5 py-1.5 text-sm focus:border-active focus:outline-none" />
          <select bind:value={minSizeUnit} class="bg-gray-800 border border-l-0 border-gray-700 rounded-r-md px-2 text-xs text-gray-400">
            <option value="B">B</option>
            <option value="KB">KB</option>
            <option value="MB">MB</option>
          </select>
        </div>
      </div>
      <div>
        <label for="max-size" class="block text-xs font-medium text-gray-500 uppercase tracking-wide mb-1.5">Max size</label>
        <div class="flex items-center">
          <input id="max-size" type="number" bind:value={maxSize} placeholder="No limit"
            class="flex-1 min-w-0 bg-gray-800 border border-gray-700 rounded-l-md px-2.5 py-1.5 text-sm focus:border-active focus:outline-none" />
          <span class="bg-gray-800 border border-l-0 border-gray-700 rounded-r-md px-2 py-1.5 text-xs text-gray-500">MB</span>
        </div>
      </div>
    </div>

    <div>
      <label for="extensions" class="block text-xs font-medium text-gray-500 uppercase tracking-wide mb-1.5">Extensions</label>
      <input id="extensions" type="text" bind:value={extensions} placeholder="e.g. py, rs, js"
        class="w-full bg-gray-800 border border-gray-700 rounded-md px-2.5 py-1.5 text-sm focus:border-active focus:outline-none placeholder-gray-600" />
    </div>

    <button
      on:click={() => showAdvanced = !showAdvanced}
      class="flex items-center gap-1.5 text-xs text-gray-500 hover:text-gray-400 transition-colors"
    >
      <span class="transition-transform" class:rotate-90={showAdvanced}>&#9654;</span>
      Advanced
    </button>

    {#if showAdvanced}
      <div class="space-y-4 pl-1 border-l border-gray-800 ml-1">
        <div>
          <label for="exclude" class="block text-xs font-medium text-gray-500 uppercase tracking-wide mb-1.5">Exclude patterns</label>
          <input id="exclude" type="text" bind:value={excludePatterns} placeholder="e.g. node_modules, .git, target"
            class="w-full bg-gray-800 border border-gray-700 rounded-md px-2.5 py-1.5 text-sm focus:border-active focus:outline-none placeholder-gray-600" />
        </div>

        <FolderPicker bind:folders={priorityPaths} label="Priority directories" />

        <div class="space-y-2">
          <label class="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
            <input type="checkbox" bind:checked={followSymlinks} class="rounded accent-active" />
            Follow symlinks
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
            <input type="checkbox" bind:checked={paranoid} class="rounded accent-active" />
            Byte-by-byte verification
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
            <input type="checkbox" bind:checked={noCache} class="rounded accent-active" />
            Disable hash cache
          </label>
        </div>
      </div>
    {/if}

    <button
      on:click={handleScan}
      disabled={roots.length === 0}
      class="w-full bg-active hover:bg-cyan-500 disabled:bg-gray-800 disabled:text-gray-600 text-white font-medium py-2.5 rounded-lg transition-colors text-sm"
    >
      Scan for duplicates
    </button>
  </div>
</div>
