<script lang="ts">
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { invoke } from "@tauri-apps/api/core";

  async function quit() {
    // Use Tauri's built-in exit via the process API
    await invoke("plugin:shell|exit", { exitCode: 0 }).catch(() => {
      // Fallback: close all windows
      window.close();
    });
  }

  async function hidePanel() {
    const win = getCurrentWebviewWindow();
    await win.hide();
  }
</script>

<div
  class="flex flex-col h-full bg-gray-900/95 backdrop-blur-xl rounded-xl border border-gray-700/50 overflow-hidden"
>
  <!-- Header -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-gray-700/50">
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full bg-green-500"></div>
      <span class="text-sm font-medium text-gray-200">Speech</span>
    </div>
    <span class="text-xs text-gray-500">v3.0.0</span>
  </div>

  <!-- Content placeholder -->
  <div class="flex-1 flex items-center justify-center p-4">
    <p class="text-sm text-gray-500">Ready</p>
  </div>

  <!-- Footer -->
  <div class="flex items-center justify-between px-3 py-2 border-t border-gray-700/50">
    <button
      onclick={hidePanel}
      class="text-xs text-gray-400 hover:text-gray-200 transition-colors"
    >
      Close
    </button>
    <button
      onclick={quit}
      class="text-xs text-red-400 hover:text-red-300 transition-colors"
    >
      Quit
    </button>
  </div>
</div>
