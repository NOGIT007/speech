<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  const barCount = 5;
  const minHeight = 8;
  const maxHeight = 80;

  let audioLevel = $state(0);
  let unlisten: (() => void) | null = null;

  onMount(async () => {
    unlisten = await listen<number>("audio-level", (event) => {
      audioLevel = event.payload;
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  function barHeight(index: number): number {
    // Center bars are taller, edges shorter (matching RecordingOverlay.swift:124-133)
    const center = (barCount - 1) / 2.0;
    const centerDistance = Math.abs(index - center) / Math.max(center, 1);
    const variation = 1.0 - centerDistance * 0.3;
    const level = audioLevel * variation;
    return minHeight + (maxHeight - minHeight) * level;
  }
</script>

<div class="flex items-center justify-center gap-1" style="height: 85px;">
  {#each Array(barCount) as _, i}
    <div
      class="rounded-sm"
      style="
        width: 12px;
        height: {barHeight(i)}px;
        background: linear-gradient(to top, #3b82f6, #06b6d4);
        transition: height 80ms ease-out;
      "
    ></div>
  {/each}
</div>
