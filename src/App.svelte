<script lang="ts">
  import "./app.css";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import MenuBarPanel from "./components/MenuBarPanel.svelte";
  import RecordingOverlay from "./overlay/RecordingOverlay.svelte";
  import SwitchOverlay from "./overlay/SwitchOverlay.svelte";
  import SettingsWindow from "./settings/SettingsWindow.svelte";

  const label = getCurrentWebviewWindow().label;
</script>

{#if label === "main"}
  <MenuBarPanel />
{:else if label === "recording-overlay"}
  <RecordingOverlay />
{:else if label === "settings"}
  <SettingsWindow />
{:else if label === "switch-overlay"}
  <SwitchOverlay />
{:else}
  <div class="p-4">
    <p class="text-sm text-gray-400">Speech v3</p>
  </div>
{/if}
