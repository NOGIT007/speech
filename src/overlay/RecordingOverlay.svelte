<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import AudioWaveform from "./AudioWaveform.svelte";

  type OverlayMode = "recording" | "processing" | "ready" | "error";

  let mode: OverlayMode = $state("recording");
  let autoPaste = $state(true);
  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    const u1 = await listen<string>("phase-changed", (event) => {
      const phase = event.payload;
      if (phase === "recording") {
        mode = "recording";
      } else if (phase === "processing") {
        mode = "processing";
      } else if (phase === "idle") {
        mode = "ready";
      }
    });
    unlisteners.push(u1);

    const u2 = await listen<string>("overlay-mode", (event) => {
      const newMode = event.payload as OverlayMode;
      if (newMode === "recording" || newMode === "processing" || newMode === "ready" || newMode === "error") {
        mode = newMode;
      }
    });
    unlisteners.push(u2);

    const u3 = await listen<boolean>("auto-paste-changed", (event) => {
      autoPaste = event.payload;
    });
    unlisteners.push(u3);

    const u4 = await listen<string>("paste-result", (event) => {
      if (event.payload === "failed") {
        mode = "error";
      }
    });
    unlisteners.push(u4);
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
  });
</script>

<div class="flex items-center justify-center w-full h-full">
  <!-- Recording glow -->
  {#if mode === "recording"}
    <div
      class="absolute inset-0 pointer-events-none"
      style="
        background: radial-gradient(ellipse at center, rgba(239,68,68,0.12) 0%, transparent 70%);
        animation: pulse-ring 2s ease-in-out infinite alternate;
      "
    ></div>
  {/if}

  <div
    class="relative flex flex-col items-center gap-4 px-10 py-7 rounded-[22px]"
    style="
      background: linear-gradient(135deg, rgba(30,30,30,0.72), rgba(20,20,20,0.78));
      backdrop-filter: blur(40px) saturate(1.4);
      -webkit-backdrop-filter: blur(40px) saturate(1.4);
      border: 1px solid rgba(255,255,255,0.08);
      box-shadow: 0 12px 40px rgba(0,0,0,0.4), inset 0 1px 0 rgba(255,255,255,0.06);
    "
  >
    <!-- All modes rendered simultaneously for smooth transitions -->
    <div class="relative" style="width: 220px; height: 140px;">
      <!-- Recording -->
      <div
        class="absolute inset-0 flex flex-col items-center justify-center gap-4"
        style="
          transition: opacity 200ms ease, transform 200ms ease;
          opacity: {mode === 'recording' ? 1 : 0};
          transform: scale({mode === 'recording' ? 1 : 0.96});
          pointer-events: {mode === 'recording' ? 'auto' : 'none'};
        "
      >
        <AudioWaveform />
        <p class="text-lg font-semibold text-white/90">Speak now</p>
        <p class="text-sm text-white/65">Release to transcribe &middot; Esc to cancel</p>
      </div>

      <!-- Processing -->
      <div
        class="absolute inset-0 flex flex-col items-center justify-center gap-4"
        style="
          transition: opacity 200ms ease, transform 200ms ease;
          opacity: {mode === 'processing' ? 1 : 0};
          transform: scale({mode === 'processing' ? 1 : 0.96});
          pointer-events: {mode === 'processing' ? 'auto' : 'none'};
        "
      >
        <div class="flex items-center justify-center" style="height: 60px;">
          <div class="comet-spinner">
            <div class="comet-glow"></div>
          </div>
        </div>
        <p class="text-lg font-semibold text-white/90">Processing...</p>
        <p class="text-sm text-white/65">
          {autoPaste ? "Will auto-paste when ready" : "Will copy to clipboard"}
        </p>
      </div>

      <!-- Ready -->
      <div
        class="absolute inset-0 flex flex-col items-center justify-center gap-4"
        style="
          transition: opacity 200ms ease, transform 200ms ease;
          opacity: {mode === 'ready' ? 1 : 0};
          transform: scale({mode === 'ready' ? 1 : 0.96});
          pointer-events: {mode === 'ready' ? 'auto' : 'none'};
        "
      >
        <div class="flex items-center justify-center" style="height: 60px;">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" fill="#22c55e" opacity="0.15" />
            <path
              d="M7 13l3 3 7-7"
              stroke="#22c55e"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              style="
                stroke-dasharray: 24;
                animation: check-draw 0.4s ease forwards;
              "
            />
          </svg>
        </div>
        <p class="text-lg font-semibold text-white/90">Ready!</p>
        <p class="text-sm text-white/65">
          {autoPaste ? "Pasted!" : "Press \u2318V to paste"}
        </p>
      </div>

      <!-- Error -->
      <div
        class="absolute inset-0 flex flex-col items-center justify-center gap-4"
        style="
          transition: opacity 200ms ease, transform 200ms ease;
          opacity: {mode === 'error' ? 1 : 0};
          transform: scale({mode === 'error' ? 1 : 0.96});
          pointer-events: {mode === 'error' ? 'auto' : 'none'};
        "
      >
        <div class="flex items-center justify-center" style="height: 60px;">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" fill="#ef4444" opacity="0.15" />
            <path d="M15 9l-6 6M9 9l6 6" stroke="#ef4444" stroke-width="2.5" stroke-linecap="round" />
          </svg>
        </div>
        <p class="text-lg font-semibold text-red-400">Paste failed</p>
        <p class="text-sm text-white/65">Text copied — press ⌘V</p>
      </div>
    </div>
  </div>
</div>

<style>
  .comet-spinner {
    position: relative;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: conic-gradient(from 0deg, transparent 60%, white);
    animation: spin 0.8s linear infinite;
    -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 3px), black calc(100% - 3px));
    mask: radial-gradient(farthest-side, transparent calc(100% - 3px), black calc(100% - 3px));
  }

  .comet-glow {
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    background: conic-gradient(from 0deg, transparent 60%, rgba(255,255,255,0.15));
    filter: blur(6px);
    animation: spin 0.8s linear infinite;
  }
</style>
