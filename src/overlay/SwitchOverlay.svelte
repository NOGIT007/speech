<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  let profileName = $state("");
  let modelId = $state("");
  let language = $state("");
  let visible = $state(false);
  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    const u1 = await listen<{
      profileName: string;
      modelId: string;
      language: string;
    }>("switch-profile", (event) => {
      profileName = event.payload.profileName;
      modelId = formatModelName(event.payload.modelId);
      language = formatLanguage(event.payload.language);
      visible = true;
    });
    unlisteners.push(u1);
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
  });

  function formatModelName(id: string): string {
    // Convert model IDs like "whisper-small" to "Small"
    const parts = id.split("-");
    if (parts.length > 1) {
      return parts
        .slice(1)
        .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
        .join(" ");
    }
    return id;
  }

  function formatLanguage(code: string): string {
    const names: Record<string, string> = {
      auto: "Auto-detect",
      en: "English",
      es: "Spanish",
      fr: "French",
      de: "German",
      it: "Italian",
      pt: "Portuguese",
      nl: "Dutch",
      pl: "Polish",
      ru: "Russian",
      ja: "Japanese",
      zh: "Chinese",
      ko: "Korean",
      da: "Danish",
      no: "Norwegian",
      sv: "Swedish",
      fi: "Finnish",
      yue: "Cantonese",
      cs: "Czech",
      uk: "Ukrainian",
      hu: "Hungarian",
      ro: "Romanian",
      bg: "Bulgarian",
      hr: "Croatian",
      sk: "Slovak",
      sl: "Slovenian",
      lt: "Lithuanian",
      lv: "Latvian",
      et: "Estonian",
      ca: "Catalan",
    };
    return names[code] || code;
  }
</script>

<div class="flex items-center justify-center w-full h-full">
  {#if visible}
    <div
      class="flex flex-col items-center gap-1.5 px-6 py-4 rounded-[14px] bg-black/60 shadow-lg shadow-black/20"
    >
      <p class="text-lg font-semibold text-white">{profileName}</p>
      <p class="text-[13px] text-white/70">
        {modelId} &middot; {language}
      </p>
    </div>
  {/if}
</div>
