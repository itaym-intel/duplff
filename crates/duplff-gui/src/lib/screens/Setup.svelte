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
    <div class="mb-10 flex justify-center">
      <img src="/logo.svg" alt="duplff" class="w-12 h-12" />
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
                class="cursor-pointer" />
              <span class="group-hover:text-text-primary transition-colors">Follow symlinks</span>
            </label>
            <label class="flex items-center gap-3 text-sm text-text-secondary cursor-pointer group">
              <input type="checkbox" bind:checked={paranoid}
                class="cursor-pointer" />
              <span class="group-hover:text-text-primary transition-colors">Byte-by-byte verification</span>
            </label>
            <label class="flex items-center gap-3 text-sm text-text-secondary cursor-pointer group">
              <input type="checkbox" bind:checked={noCache}
                class="cursor-pointer" />
              <span class="group-hover:text-text-primary transition-colors">Disable hash cache</span>
            </label>
          </div>
        </div>
      {/if}

      <!-- Scan button -->
      <div class="pt-4 flex justify-center">
        <button
          onclick={handleScan}
          disabled={roots.length === 0}
          class="scan-btn group relative flex items-center gap-3 px-10 py-3.5 rounded-2xl text-sm font-semibold
            tracking-wide uppercase transition-all duration-300 ease-out
            {roots.length > 0
              ? 'scan-btn--active text-white'
              : 'bg-surface/60 text-text-muted border border-dashed border-border cursor-not-allowed'}"
        >
          {#if roots.length > 0}
            <span class="relative z-10">Scan</span>
            <span class="relative z-10 transition-transform duration-300 group-hover:translate-x-1">&rarr;</span>
          {:else}
            <span>Scan</span>
          {/if}
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

  .scan-btn--active {
    background: linear-gradient(
      180deg,
      oklch(0.50 0.13 130.7) 0%,
      oklch(0.447 0.122 130.7) 50%,
      oklch(0.40 0.11 130.7) 100%
    );
    box-shadow:
      0 1px 0 0 oklch(0.55 0.10 130.7 / 0.3) inset,
      0 8px 20px -4px oklch(0.447 0.122 130.7 / 0.35),
      0 2px 6px -1px oklch(0.447 0.122 130.7 / 0.2);
  }
  .scan-btn--active:hover {
    transform: translateY(-2px);
    box-shadow:
      0 1px 0 0 oklch(0.55 0.10 130.7 / 0.4) inset,
      0 14px 28px -4px oklch(0.447 0.122 130.7 / 0.4),
      0 4px 10px -2px oklch(0.447 0.122 130.7 / 0.25);
  }
  .scan-btn--active:active {
    transform: translateY(0px) scale(0.98);
    box-shadow:
      0 1px 0 0 oklch(0.55 0.10 130.7 / 0.2) inset,
      0 4px 8px -2px oklch(0.447 0.122 130.7 / 0.3);
  }
</style>
