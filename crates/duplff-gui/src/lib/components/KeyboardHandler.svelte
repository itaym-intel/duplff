<script lang="ts">
  import { focusedGroup, expandedGroups, markedFiles } from '$lib/stores';
  import type { DuplicateGroup } from '$lib/types';

  let {
    groups,
    onTrash,
    onShowHelp,
    onFocusFilter,
  }: {
    groups: { group: DuplicateGroup; index: number }[];
    onTrash: () => void;
    onShowHelp: () => void;
    onFocusFilter: () => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if ((e.target as HTMLElement)?.tagName === 'INPUT' ||
        (e.target as HTMLElement)?.tagName === 'SELECT') {
      if (e.key === 'Escape') {
        (e.target as HTMLElement).blur();
        e.preventDefault();
      }
      return;
    }

    switch (e.key) {
      case 'j':
      case 'ArrowDown':
        e.preventDefault();
        focusedGroup.update(i => {
          const idx = groups.findIndex(g => g.index === i);
          if (idx < groups.length - 1) return groups[idx + 1].index;
          return i;
        });
        break;

      case 'k':
      case 'ArrowUp':
        e.preventDefault();
        focusedGroup.update(i => {
          const idx = groups.findIndex(g => g.index === i);
          if (idx > 0) return groups[idx - 1].index;
          return i;
        });
        break;

      case 'Enter':
      case 'ArrowRight':
        e.preventDefault();
        expandedGroups.update(set => {
          const next = new Set(set);
          let current: number;
          focusedGroup.subscribe(v => current = v)();
          if (next.has(current!)) next.delete(current!);
          else next.add(current!);
          return next;
        });
        break;

      case 'Escape':
      case 'ArrowLeft':
        e.preventDefault();
        expandedGroups.update(set => {
          const next = new Set(set);
          let current: number;
          focusedGroup.subscribe(v => current = v)();
          next.delete(current!);
          return next;
        });
        break;

      case ' ':
        e.preventDefault();
        {
          let current: number;
          focusedGroup.subscribe(v => current = v)();
          const item = groups.find(g => g.index === current!);
          if (item) {
            markedFiles.update(set => {
              const next = new Set(set);
              const paths = item.group.duplicates.map(d => d.entry.path);
              const allMarked = paths.every(p => next.has(p));
              for (const p of paths) {
                if (allMarked) next.delete(p);
                else next.add(p);
              }
              return next;
            });
          }
        }
        break;

      case 'd':
      case 'Delete':
        e.preventDefault();
        onTrash();
        break;

      case '/':
        e.preventDefault();
        onFocusFilter();
        break;

      case '?':
        e.preventDefault();
        onShowHelp();
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />
