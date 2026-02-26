<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { TranscriptionItem } from "../lib/types";

  interface Props {
    item: TranscriptionItem;
    ondelete: () => void;
  }

  let { item, ondelete }: Props = $props();

  async function copyToClipboard() {
    try {
      await invoke("copy_to_clipboard", { text: item.text });
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }

  function timeAgo(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (seconds < 60) return "just now";
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }
</script>

<div
  class="group flex items-center gap-1 px-3 py-1.5 rounded-lg hover:bg-white/5 transition-colors"
>
  <button
    class="flex-1 flex items-start gap-2 text-left min-w-0"
    onclick={copyToClipboard}
    title="Click to copy"
  >
    <div class="flex-1 min-w-0">
      <p class="text-xs text-white/80 line-clamp-2">{item.preview}</p>
      <p class="text-[10px] text-white/30 mt-0.5">{timeAgo(item.timestamp)}</p>
    </div>
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 16 16"
      fill="currentColor"
      class="w-3.5 h-3.5 text-white/20 group-hover:text-white/50 transition-colors mt-0.5 shrink-0"
    >
      <path d="M5.5 3.5A1.5 1.5 0 017 2h4.5A1.5 1.5 0 0113 3.5v9a1.5 1.5 0 01-1.5 1.5H7A1.5 1.5 0 015.5 12.5v-9z" />
      <path d="M3 5.5A1.5 1.5 0 014.5 4H5v8.5A2.5 2.5 0 007.5 15h4v.5A1.5 1.5 0 0110 17H4.5A1.5 1.5 0 013 15.5v-10z" />
    </svg>
  </button>
  <button
    class="text-white/20 hover:text-red-400 transition-colors shrink-0"
    onclick={ondelete}
    title="Delete"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 16 16"
      fill="currentColor"
      class="w-3.5 h-3.5"
    >
      <path d="M5.28 4.22a.75.75 0 00-1.06 1.06L6.94 8l-2.72 2.72a.75.75 0 101.06 1.06L8 9.06l2.72 2.72a.75.75 0 101.06-1.06L9.06 8l2.72-2.72a.75.75 0 00-1.06-1.06L8 6.94 5.28 4.22z" />
    </svg>
  </button>
</div>
