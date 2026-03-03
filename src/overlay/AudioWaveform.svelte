<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  const BAR_COUNT = 80;
  const BAR_WIDTH = 2;
  const BAR_GAP = 1.5;
  const MIN_HEIGHT = 2;
  const MAX_HEIGHT = 48;
  const CANVAS_HEIGHT = 56;

  // Rolling buffer: newest sample at the center, older samples toward edges
  let samples: number[] = $state(new Array(BAR_COUNT).fill(0));
  let unlisten: (() => void) | null = null;

  onMount(async () => {
    unlisten = await listen<number>("audio-level", (event) => {
      // Push new sample and shift oldest off the front
      samples = [...samples.slice(1), event.payload];
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  // Map a sample index to bar height. The buffer stores oldest-first,
  // so index BAR_COUNT-1 is the newest (center). We mirror from center outward.
  function getBarHeight(visualIndex: number): number {
    // visualIndex 0 = leftmost, BAR_COUNT-1 = rightmost
    // center = newest sample, edges = oldest
    const center = (BAR_COUNT - 1) / 2;
    const distFromCenter = Math.abs(visualIndex - center);
    // Map visual position back to buffer index:
    // center -> last element (newest), edges -> first elements (oldest)
    const bufferIndex = Math.round(BAR_COUNT - 1 - distFromCenter);
    const level = samples[bufferIndex] ?? 0;
    const boosted = Math.sqrt(level);
    return MIN_HEIGHT + (MAX_HEIGHT - MIN_HEIGHT) * boosted;
  }

  function barOpacity(visualIndex: number): number {
    const center = (BAR_COUNT - 1) / 2;
    const distFromCenter = Math.abs(visualIndex - center);
    const normalized = distFromCenter / center;
    // Fade out toward edges
    return 1.0 - normalized * 0.6;
  }
</script>

<div class="relative" style="height: {CANVAS_HEIGHT}px; width: {BAR_COUNT * (BAR_WIDTH + BAR_GAP)}px;">
  <!-- Dotted baseline -->
  <div
    class="absolute left-0 right-0"
    style="
      top: 50%;
      border-top: 1px dashed rgba(255,255,255,0.15);
    "
  ></div>

  <!-- Bars centered vertically -->
  <div class="flex items-center justify-center h-full" style="gap: {BAR_GAP}px;">
    {#each Array(BAR_COUNT) as _, i}
      {@const h = getBarHeight(i)}
      <div
        style="
          width: {BAR_WIDTH}px;
          height: {h}px;
          background: rgba(255,255,255,{barOpacity(i) * 0.9});
          border-radius: 1px;
          transition: height 50ms linear;
        "
      ></div>
    {/each}
  </div>
</div>
