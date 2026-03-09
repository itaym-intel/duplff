<script lang="ts">
  import type { DuplicateGroup } from '$lib/types';
  import GroupCard from './GroupCard.svelte';
  import { expandedGroups, focusedGroup, ignoredGroups } from '$lib/stores';

  let {
    groups,
  }: {
    groups: { group: DuplicateGroup; index: number }[];
  } = $props();

  const COLLAPSED_HEIGHT = 72;
  const EXPANDED_HEIGHT = 400;
  const BUFFER = 5;

  let container: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let containerHeight = $state(700);

  function getItemHeight(i: number): number {
    const item = groups[i];
    if (!item) return COLLAPSED_HEIGHT;
    return $expandedGroups.has(item.index) ? EXPANDED_HEIGHT : COLLAPSED_HEIGHT;
  }

  let totalHeight = $derived(
    groups.reduce((sum, _, i) => sum + getItemHeight(i) + 8, 0)
  );

  let visibleRange = $derived.by(() => {
    let offset = 0;
    let start = 0;
    let end = 0;

    for (let i = 0; i < groups.length; i++) {
      const h = getItemHeight(i) + 8;
      if (offset + h > scrollTop - BUFFER * COLLAPSED_HEIGHT) {
        start = i;
        break;
      }
      offset += h;
    }

    offset = 0;
    for (let i = 0; i < groups.length; i++) {
      offset += getItemHeight(i) + 8;
      if (offset > scrollTop + containerHeight + BUFFER * COLLAPSED_HEIGHT) {
        end = i + 1;
        break;
      }
    }
    if (end === 0) end = groups.length;

    return { start: Math.max(0, start), end: Math.min(groups.length, end) };
  });

  let offsetTop = $derived.by(() => {
    let offset = 0;
    for (let i = 0; i < visibleRange.start; i++) {
      offset += getItemHeight(i) + 8;
    }
    return offset;
  });

  let visibleItems = $derived(
    groups.slice(visibleRange.start, visibleRange.end)
  );

  function handleScroll() {
    if (container) {
      scrollTop = container.scrollTop;
      containerHeight = container.clientHeight;
    }
  }

  function toggleExpand(index: number) {
    expandedGroups.update(set => {
      const next = new Set(set);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return next;
    });
  }

  function ignoreGroup(index: number) {
    ignoredGroups.update(set => {
      const next = new Set(set);
      next.add(index);
      return next;
    });
  }

  $effect(() => {
    if (container) {
      containerHeight = container.clientHeight;
    }
  });
</script>

<div
  bind:this={container}
  onscroll={handleScroll}
  class="flex-1 overflow-auto"
>
  <div style="height: {totalHeight}px; position: relative;">
    <div style="transform: translateY({offsetTop}px);" class="space-y-2 px-4 py-2">
      {#each visibleItems as { group, index } (index)}
        <GroupCard
          {group}
          {index}
          expanded={$expandedGroups.has(index)}
          focused={$focusedGroup === index}
          onToggleExpand={() => toggleExpand(index)}
          onIgnore={() => ignoreGroup(index)}
        />
      {/each}
    </div>
  </div>
</div>
