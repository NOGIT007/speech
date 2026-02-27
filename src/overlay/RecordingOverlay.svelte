<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import AudioWaveform from "./AudioWaveform.svelte";

  type OverlayMode = "recording" | "processing" | "ready";

  let mode: OverlayMode = $state("recording");
  let autoPaste = $state(true);
  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    // Listen for phase changes to update overlay mode
    const u1 = await listen<string>("phase-changed", (event) => {
      const phase = event.payload;
      if (phase === "recording") {
        mode = "recording";
      } else if (phase === "processing") {
        mode = "processing";
      } else if (phase === "idle") {
        // Show ready briefly then the window will be hidden by Rust
        mode = "ready";
      }
    });
    unlisteners.push(u1);

    // Listen for overlay mode changes (direct control from Rust)
    const u2 = await listen<string>("overlay-mode", (event) => {
      const newMode = event.payload as OverlayMode;
      if (newMode === "recording" || newMode === "processing" || newMode === "ready") {
        mode = newMode;
      }
    });
    unlisteners.push(u2);

    // Listen for auto-paste setting changes
    const u3 = await listen<boolean>("auto-paste-changed", (event) => {
      autoPaste = event.payload;
    });
    unlisteners.push(u3);
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
  });
</script>

<div class="flex items-center justify-center w-full h-full">
  <div
    class="flex flex-col items-center gap-4 px-10 py-7 rounded-[22px]"
    style="background: rgba(0, 0, 0, 0.65); box-shadow: 0 12px 24px rgba(0, 0, 0, 0.25);"
  >
    {#if mode === "recording"}
      <AudioWaveform />

      <p class="text-lg font-semibold text-white/90">
        Recording...
      </p>

      <p class="text-sm text-white/55">
        Release to transcribe &middot; Esc to cancel
      </p>

    {:else if mode === "processing"}
      <div class="flex items-center justify-center" style="height: 105px;">
        <div class="spinner"></div>
      </div>

      <p class="text-lg font-semibold text-white/90">
        Processing...
      </p>

      <p class="text-sm text-white/55">
        {autoPaste ? "Will auto-paste when ready" : "Will copy to clipboard"}
      </p>

    {:else if mode === "ready"}
      <div class="flex items-center justify-center" style="height: 105px;">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#22c55e" class="w-12 h-12">
          <path fill-rule="evenodd" d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm13.36-1.814a.75.75 0 10-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z" clip-rule="evenodd" />
        </svg>
      </div>

      <p class="text-lg font-semibold text-white/90">
        Ready!
      </p>

      <p class="text-sm text-white/55">
        {autoPaste ? "Pasted!" : "Press \u2318V to paste"}
      </p>
    {/if}
  </div>
</div>

<style>
  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.2);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
