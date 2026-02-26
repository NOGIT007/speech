<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import TranscriptionRow from "./TranscriptionRow.svelte";
  import type { AppPhase, TranscriptionItem, Settings } from "../lib/types";

  let phase = $state<AppPhase>("idle");
  let history = $state<TranscriptionItem[]>([]);
  let errorMessage = $state<string | null>(null);
  let recordHotkey = $state("Alt+Space");
  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    await refreshState();

    // Load hotkey display
    try {
      const settings = (await invoke("get_settings")) as Settings;
      recordHotkey = settings.recordHotkey;
    } catch {
      // use default
    }

    // Listen for phase changes
    const u1 = await listen<string>("phase-changed", (event) => {
      phase = event.payload as AppPhase;
    });
    unlisteners.push(u1);

    // Listen for history updates
    const u2 = await listen("history-updated", async () => {
      await refreshHistory();
    });
    unlisteners.push(u2);

    // Listen for errors
    const u3 = await listen<string>("transcription-error", (event) => {
      errorMessage = event.payload;
    });
    unlisteners.push(u3);
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
  });

  async function refreshState() {
    try {
      phase = (await invoke("get_phase")) as AppPhase;
    } catch {
      // ignore
    }
    await refreshHistory();
  }

  async function refreshHistory() {
    try {
      history = (await invoke("get_history")) as TranscriptionItem[];
    } catch {
      // ignore
    }
  }

  async function clearHistory() {
    try {
      await invoke("clear_history");
      history = [];
    } catch (e) {
      console.error("Failed to clear history:", e);
    }
  }

  async function deleteHistoryItem(id: string) {
    try {
      await invoke("delete_history_item", { id });
      history = history.filter((h) => h.id !== id);
    } catch (e) {
      console.error("Failed to delete item:", e);
    }
  }

  async function openSettings() {
    try {
      await invoke("open_settings");
    } catch (e) {
      console.error("Failed to open settings:", e);
    }
  }

  async function relaunch() {
    try {
      await invoke("relaunch_app");
    } catch (e) {
      console.error("Failed to relaunch:", e);
    }
  }

  async function quit() {
    await invoke("plugin:shell|exit", { exitCode: 0 }).catch(() => {
      window.close();
    });
  }

  function statusText(): string {
    switch (phase) {
      case "recording":
        return "Recording...";
      case "processing":
        return "Transcribing...";
      default:
        return "Ready";
    }
  }

  function statusColor(): string {
    switch (phase) {
      case "recording":
        return "bg-red-500";
      case "processing":
        return "bg-orange-500";
      default:
        return "bg-green-500";
    }
  }

  function statusGlow(): string {
    switch (phase) {
      case "recording":
        return "shadow-red-500/50";
      case "processing":
        return "shadow-orange-500/50";
      default:
        return "shadow-green-500/50";
    }
  }

  function formatHotkey(s: string): string {
    return s
      .replace("Alt", "\u2325")
      .replace("Ctrl", "\u2303")
      .replace("Shift", "\u21E7")
      .replace("Super", "\u2318")
      .replace("Space", "\u2423")
      .replace(/\+/g, "");
  }
</script>

<div class="flex flex-col h-full bg-[#1e1e1e] text-white overflow-hidden">
  <!-- Status header -->
  <div class="px-3 py-3">
    <div class="flex items-center gap-2.5">
      <div
        class="w-2.5 h-2.5 rounded-full {statusColor()} shadow-sm {statusGlow()}"
      ></div>
      <div>
        <p class="text-[13px] font-semibold">{statusText()}</p>
        <p class="text-[11px] text-white/40">
          Hold {formatHotkey(recordHotkey)} to dictate
        </p>
      </div>
    </div>
  </div>

  <div class="h-px bg-white/10"></div>

  <!-- Transcription history -->
  {#if history.length > 0}
    <div class="flex-1 overflow-y-auto">
      <div class="flex items-center justify-between px-3 pt-2 pb-1">
        <span class="text-[11px] font-medium text-white/40">Recent</span>
        {#if history.length > 1}
          <button
            class="text-[10px] text-white/30 hover:text-white/50 transition-colors"
            onclick={clearHistory}
          >
            Clear
          </button>
        {/if}
      </div>
      {#each history as item (item.id)}
        <TranscriptionRow
          {item}
          ondelete={() => deleteHistoryItem(item.id)}
        />
      {/each}
    </div>

    <div class="h-px bg-white/10"></div>
  {/if}

  <!-- Error message -->
  {#if errorMessage}
    <div class="px-3 py-2">
      <div class="flex items-start gap-2">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          class="w-4 h-4 text-red-400 shrink-0 mt-0.5"
        >
          <path
            fill-rule="evenodd"
            d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
            clip-rule="evenodd"
          />
        </svg>
        <p class="text-[11px] text-red-400 line-clamp-2">{errorMessage}</p>
      </div>
    </div>
    <div class="h-px bg-white/10"></div>
  {/if}

  <!-- Actions -->
  <div class="px-3 py-1.5">
    <button
      class="w-full text-left text-[13px] text-white/70 hover:text-white hover:bg-white/5 px-2 py-1.5 rounded transition-colors"
      onclick={openSettings}
    >
      Settings...
    </button>
  </div>

  <div class="h-px bg-white/10"></div>

  <!-- Relaunch / Quit -->
  <div class="flex items-center gap-2 px-3 py-2">
    <button
      class="flex-1 flex items-center justify-center gap-1.5 text-[11px] text-white/50 hover:text-white/70 bg-white/5 hover:bg-white/10 rounded-md py-1.5 transition-colors"
      onclick={relaunch}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 16 16"
        fill="currentColor"
        class="w-3 h-3"
      >
        <path
          fill-rule="evenodd"
          d="M13.836 2.477a.75.75 0 01.75.75v3.182a.75.75 0 01-.75.75h-3.182a.75.75 0 010-1.5h1.37l-.84-.841a4.5 4.5 0 10.315 6.644.75.75 0 011.119.996A6 6 0 1112.803 3.57l.836.837V2.727a.75.75 0 01.75-.75z"
          clip-rule="evenodd"
        />
      </svg>
      Relaunch
    </button>
    <button
      class="flex-1 flex items-center justify-center gap-1.5 text-[11px] text-white/50 hover:text-white/70 bg-white/5 hover:bg-white/10 rounded-md py-1.5 transition-colors"
      onclick={quit}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 16 16"
        fill="currentColor"
        class="w-3 h-3"
      >
        <path d="M5.28 4.22a.75.75 0 00-1.06 1.06L6.94 8l-2.72 2.72a.75.75 0 101.06 1.06L8 9.06l2.72 2.72a.75.75 0 101.06-1.06L9.06 8l2.72-2.72a.75.75 0 00-1.06-1.06L8 6.94 5.28 4.22z" />
      </svg>
      Quit
    </button>
  </div>

  <!-- Version -->
  <div class="px-3 py-2 text-center">
    <span class="text-[10px] text-white/20">Speech v3.0.0</span>
  </div>
</div>
