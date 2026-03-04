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

<div class="max-w-2xl mx-auto p-8">
  <h1 class="text-3xl font-bold mb-2">duplff</h1>
  <p class="text-gray-400 mb-8">Find and remove duplicate files</p>

  <div class="space-y-6">
    <FolderPicker bind:folders={roots} label="Scan directories" />

    <div>
      <label class="text-sm font-medium text-gray-300">Extension filter</label>
      <input
        type="text"
        bind:value={extensions}
        placeholder="e.g. py,rs,js (leave empty for all)"
        class="mt-1 w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none"
      />
    </div>

    <div>
      <label class="text-sm font-medium text-gray-300">Exclude patterns</label>
      <input
        type="text"
        bind:value={excludePatterns}
        placeholder="e.g. node_modules,.git,target"
        class="mt-1 w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none"
      />
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="text-sm font-medium text-gray-300">Min size</label>
        <div class="flex gap-2 mt-1">
          <input type="number" bind:value={minSize} min="0"
            class="flex-1 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none" />
          <select bind:value={minSizeUnit} class="bg-gray-800 border border-gray-700 rounded px-2 text-sm">
            <option value="B">B</option>
            <option value="KB">KB</option>
            <option value="MB">MB</option>
          </select>
        </div>
      </div>
      <div>
        <label class="text-sm font-medium text-gray-300">Max size (MB, optional)</label>
        <input type="number" bind:value={maxSize} placeholder="No limit"
          class="mt-1 w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none" />
      </div>
    </div>

    <FolderPicker bind:folders={priorityPaths} label="Priority directories (optional)" />

    <div class="space-y-2">
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={followSymlinks} class="rounded" />
        <span>Follow symlinks</span>
      </label>
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={paranoid} class="rounded" />
        <span>Paranoid mode (byte-by-byte verification)</span>
      </label>
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={noCache} class="rounded" />
        <span>Disable hash cache</span>
      </label>
    </div>

    <button
      on:click={handleScan}
      disabled={roots.length === 0}
      class="w-full bg-active hover:bg-cyan-500 disabled:bg-gray-700 disabled:text-gray-500 text-white font-medium py-3 rounded-lg transition-colors"
    >
      Scan for duplicates
    </button>
  </div>
</div>
