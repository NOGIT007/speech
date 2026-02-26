<script lang="ts">
  import "./app.css";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  const label = getCurrentWebviewWindow().label;
</script>

{#if label === "main"}
  <div class="p-4">
    <p class="text-sm text-gray-400">Speech v3 — menu bar panel</p>
  </div>
{:else if label === "recording-overlay"}
  <div class="p-4">
    <p class="text-sm text-gray-400">Recording overlay</p>
  </div>
{:else if label === "switch-overlay"}
  <div class="p-4">
    <p class="text-sm text-gray-400">Switch overlay</p>
  </div>
{:else}
  <div class="p-4">
    <p class="text-sm text-gray-400">Speech v3</p>
  </div>
{/if}
