<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  const barCount = 5;
  const minHeight = 8;
  const maxHeight = 64;

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
    const center = (barCount - 1) / 2.0;
    const centerDistance = Math.abs(index - center) / Math.max(center, 1);
    const variation = 1.0 - Math.pow(centerDistance, 2) * 0.5;
    // Square root curve: small audio levels still produce tall bars
    const boosted = Math.sqrt(audioLevel);
    const level = boosted * variation;
    return minHeight + (maxHeight - minHeight) * level;
  }

  function barGradient(): string {
    if (audioLevel > 0.7) {
      return "linear-gradient(to top, #3b82f6, #06b6d4, #e0f2fe)";
    }
    return "linear-gradient(to top, #3b82f6, #06b6d4)";
  }
</script>

<div class="flex items-end justify-center gap-[5px]" style="height: 64px;">
  {#each Array(barCount) as _, i}
    <div
      class="rounded-full"
      style="
        width: 4px;
        height: {barHeight(i)}px;
        background: {barGradient()};
        box-shadow: 0 0 8px rgba(59,130,246,0.3);
        transition: height 60ms cubic-bezier(0.33, 1, 0.68, 1);
      "
    ></div>
  {/each}
</div>
