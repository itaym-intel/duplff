<script lang="ts">
  import { scanConfig, currentScreen } from '$lib/stores';
  import FolderPicker from '$lib/components/FolderPicker.svelte';

  let roots: string[] = $state([]);
  let priorityPaths: string[] = $state([]);
  let extensions = $state('');
  let excludePatterns = $state('');
  let minSize = $state(1);
  let minSizeUnit: 'B' | 'KB' | 'MB' = $state('B');
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
  <div class="w-full max-w-lg">
    <!-- Header -->
    <div class="mb-10">
      <div class="flex items-center gap-3">
        <img src="/logo.svg" alt="duplff" class="w-8 h-8" />
        <h1 class="text-xl font-semibold tracking-tight">duplff</h1>
      </div>
    </div>

    <div class="space-y-5">
      <!-- Folder Picker -->
      <FolderPicker bind:folders={roots} label="Scan directories" />

      <!-- Advanced toggle -->
      <button
        onclick={() => showAdvanced = !showAdvanced}
        class="flex items-center gap-2.5 text-sm text-text-muted hover:text-text-secondary transition-colors py-1"
      >
        <span class="text-[10px] transition-transform duration-150 {showAdvanced ? 'rotate-90' : ''}">&#9654;</span>
        Advanced options
      </button>

      {#if showAdvanced}
        <div class="space-y-5 pl-4 border-l-2 border-border-subtle ml-1 animate-[fadeIn_150ms_ease-out]">
          <!-- Size filters -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label for="min-size" class="block text-[11px] font-medium text-text-muted uppercase tracking-widest mb-2">Min size</label>
              <div class="flex">
                <input id="min-size" type="number" bind:value={minSize} min="0"
                  class="flex-1 min-w-0 bg-surface border border-border rounded-l-lg px-3 py-2 text-sm text-text-primary
                    focus:border-active/40 focus:outline-none focus:ring-1 focus:ring-active/15 transition-shadow" />
                <select bind:value={minSizeUnit}
                  class="bg-surface border border-l-0 border-border rounded-r-lg px-2.5 text-sm text-text-muted cursor-pointer
                    focus:outline-none">
                  <option value="B">B</option>
                  <option value="KB">KB</option>
                  <option value="MB">MB</option>
                </select>
              </div>
            </div>
            <div>
              <label for="max-size" class="block text-[11px] font-medium text-text-muted uppercase tracking-widest mb-2">Max size</label>
              <div class="flex items-center">
                <input id="max-size" type="number" bind:value={maxSize} placeholder="No limit"
                  class="flex-1 min-w-0 bg-surface border border-border rounded-l-lg px-3 py-2 text-sm text-text-primary
                    focus:border-active/40 focus:outline-none focus:ring-1 focus:ring-active/15 placeholder-text-muted transition-shadow" />
                <span class="bg-surface border border-l-0 border-border rounded-r-lg px-2.5 py-2 text-sm text-text-muted select-none">MB</span>
              </div>
            </div>
          </div>

          <!-- Extensions -->
          <div>
            <label for="extensions" class="block text-[11px] font-medium text-text-muted uppercase tracking-widest mb-2">File extensions</label>
            <input id="extensions" type="text" bind:value={extensions} placeholder="Leave empty for all — or: py, rs, js"
              class="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm text-text-primary
                focus:border-active/40 focus:outline-none focus:ring-1 focus:ring-active/15 placeholder-text-muted transition-shadow" />
          </div>

          <div>
            <label for="exclude" class="block text-[11px] font-medium text-text-muted uppercase tracking-widest mb-2">Exclude patterns</label>
            <input id="exclude" type="text" bind:value={excludePatterns} placeholder="e.g. node_modules, .git, target"
              class="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm text-text-primary
                focus:border-active/40 focus:outline-none focus:ring-1 focus:ring-active/15 placeholder-text-muted transition-shadow" />
          </div>

          <FolderPicker bind:folders={priorityPaths} label="Priority directories" />

          <div class="space-y-3">
            <label class="flex items-center gap-3 text-sm text-text-secondary cursor-pointer group">
              <input type="checkbox" bind:checked={followSymlinks}
                class="w-4 h-4 rounded accent-active cursor-pointer" />
              <span class="group-hover:text-text-primary transition-colors">Follow symlinks</span>
            </label>
            <label class="flex items-center gap-3 text-sm text-text-secondary cursor-pointer group">
              <input type="checkbox" bind:checked={paranoid}
                class="w-4 h-4 rounded accent-active cursor-pointer" />
              <span class="group-hover:text-text-primary transition-colors">Byte-by-byte verification</span>
            </label>
            <label class="flex items-center gap-3 text-sm text-text-secondary cursor-pointer group">
              <input type="checkbox" bind:checked={noCache}
                class="w-4 h-4 rounded accent-active cursor-pointer" />
              <span class="group-hover:text-text-primary transition-colors">Disable hash cache</span>
            </label>
          </div>
        </div>
      {/if}

      <!-- Scan button -->
      <div class="pt-2">
        <button
          onclick={handleScan}
          disabled={roots.length === 0}
          class="w-full py-3 rounded-xl text-sm font-medium transition-all duration-200
            {roots.length > 0
              ? 'text-white shadow-lg hover:brightness-110 active:scale-[0.99]'
              : 'bg-surface text-text-muted border border-border cursor-not-allowed'}"
          style={roots.length > 0 ? 'background: oklch(0.447 0.122 130.7); box-shadow: 0 10px 15px -3px oklch(0.447 0.122 130.7 / 0.25);' : ''}
        >
          Scan
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
