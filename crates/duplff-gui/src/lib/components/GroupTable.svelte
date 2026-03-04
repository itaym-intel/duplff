<script lang="ts">
  import type { DuplicateGroup, SortMode } from '$lib/types';
  import { formatBytes, truncatePath } from '$lib/format';

  export let groups: DuplicateGroup[];
  export let sortMode: SortMode;
  export let filterText: string;
  export let onSelectGroup: (index: number) => void;

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

<div class="overflow-auto">
  <table class="w-full text-sm">
    <thead class="text-gray-400 text-left border-b border-gray-700">
      <tr>
        <th class="py-2 px-3 w-12">#</th>
        <th class="py-2 px-3">Files</th>
        <th class="py-2 px-3">Size</th>
        <th class="py-2 px-3">Wasted</th>
        <th class="py-2 px-3">Sample Path</th>
      </tr>
    </thead>
    <tbody>
      {#each sortedGroups as { group, index }, i}
        <tr
          class="border-b border-gray-800 hover:bg-gray-800/50 cursor-pointer transition-colors"
          on:click={() => onSelectGroup(index)}
        >
          <td class="py-2 px-3 text-gray-500">{i + 1}</td>
          <td class="py-2 px-3 font-mono">{group.duplicates.length + 1}</td>
          <td class="py-2 px-3 font-mono">{formatBytes(group.size)}</td>
          <td class="py-2 px-3 font-mono text-delete">{formatBytes(group.size * group.duplicates.length)}</td>
          <td class="py-2 px-3 font-mono truncate max-w-xs" title={group.keep.entry.path}>
            {truncatePath(group.keep.entry.path)}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
