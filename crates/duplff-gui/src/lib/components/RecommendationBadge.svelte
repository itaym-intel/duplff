<script lang="ts">
  import type { Confidence } from '$lib/types';
  import { confidenceColor } from '$lib/format';

  let { reason, confidence }: { reason: string; confidence: Confidence } = $props();

  const reasonDetails: Record<string, { label: string; icon: string }> = {
    PriorityPath: { label: 'Inside priority directory', icon: '~' },
    DeepestPath: { label: 'Deepest path', icon: '/' },
    NewestModification: { label: 'Newest modification date', icon: '+' },
    LexicographicFirst: { label: 'First alphabetically', icon: 'a' },
  };

  let detail = $derived(reasonDetails[reason] || { label: reason, icon: '?' });
  let color = $derived(confidenceColor(confidence));
</script>

<div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs">
  <span class="flex items-center gap-1 text-keep">
    <span class="font-mono text-[10px]">{detail.icon}</span>
    {detail.label}
  </span>
  <span class="flex items-center gap-1.5 {color}">
    Confidence:
    <span class="font-medium capitalize">{confidence}</span>
  </span>
</div>
