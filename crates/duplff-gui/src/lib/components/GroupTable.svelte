<script lang="ts">
  import type { DuplicateGroup, SortMode } from '$lib/types';
  import { formatBytes, truncatePath } from '$lib/format';

  export let groups: DuplicateGroup[];
  export let sortMode: SortMode;
  export let filterText: string;
  export let onSelectGroup: (index: number) => void;

  $: maxWasted = Math.max(...groups.map(g => g.size * g.duplicates.length), 1);

  $: filteredGroups = groups
    .map((g, i) => ({ group: g, index: i }))
    .filter(({ group }) => {
      if (!filterText) return true;
      const lower = filterText.toLowerCase();
      return group.keep.entry.path.toLowerCase().includes(lower) ||
        group.duplicates.some(d => d.entry.path.toLowerCase().includes(lower));
    });

  $: sortedGroups = [...filteredGroups].sort((a, b) => {
    switch (sortMode) {
      case 'wasted':
        return (b.group.size * b.group.duplicates.length) - (a.group.size * a.group.duplicates.length);
      case 'size':
        return b.group.size - a.group.size;
      case 'files':
        return (b.group.duplicates.length + 1) - (a.group.duplicates.length + 1);
      case 'path':
        return a.group.keep.entry.path.localeCompare(b.group.keep.entry.path);
      default:
        return 0;
    }
  });
</script>

<table class="w-full text-xs">
  <thead>
    <tr class="text-gray-600 text-left">
      <th class="py-1.5 px-4 font-medium w-14">Files</th>
      <th class="py-1.5 px-4 font-medium w-20">Size</th>
      <th class="py-1.5 px-4 font-medium w-32">Wasted</th>
      <th class="py-1.5 px-4 font-medium">Path</th>
    </tr>
  </thead>
  <tbody>
    {#each sortedGroups as { group, index }, i}
      {@const wasted = group.size * group.duplicates.length}
      {@const wastedPct = (wasted / maxWasted) * 100}
      <tr
        class="border-t border-gray-800/50 hover:bg-gray-800/30 cursor-pointer transition-colors {i % 2 === 0 ? '' : 'bg-gray-800/10'}"
        on:click={() => onSelectGroup(index)}
      >
        <td class="py-1.5 px-4 font-mono text-gray-400">{group.duplicates.length + 1}</td>
        <td class="py-1.5 px-4 font-mono text-gray-400">{formatBytes(group.size)}</td>
        <td class="py-1.5 px-4">
          <div class="flex items-center gap-2">
            <div class="flex-1 h-1 bg-gray-800 rounded-full overflow-hidden">
              <div class="h-full bg-delete/60 rounded-full" style="width: {wastedPct}%"></div>
            </div>
            <span class="font-mono text-delete text-[11px] w-16 text-right">{formatBytes(wasted)}</span>
          </div>
        </td>
        <td class="py-1.5 px-4 font-mono text-gray-500 truncate max-w-xs" title={group.keep.entry.path}>
          {truncatePath(group.keep.entry.path)}
        </td>
      </tr>
    {/each}
  </tbody>
</table>
