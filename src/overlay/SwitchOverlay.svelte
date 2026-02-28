<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  let profileName = $state("");
  let modelId = $state("");
  let language = $state("");
  let visible = $state(false);
  let unlisteners: (() => void)[] = [];

  const palette = ["#3b82f6", "#22c55e", "#f59e0b", "#ef4444", "#a855f7", "#06b6d4"];

  function profileColor(name: string): string {
    let sum = 0;
    for (let i = 0; i < name.length; i++) {
      sum += name.charCodeAt(i);
    }
    return palette[sum % palette.length];
  }

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
      class="switch-enter flex flex-col items-center gap-1.5 px-6 py-4 rounded-[14px]"
      style="
        background: linear-gradient(135deg, rgba(30,30,30,0.72), rgba(20,20,20,0.78));
        backdrop-filter: blur(40px) saturate(1.4);
        -webkit-backdrop-filter: blur(40px) saturate(1.4);
        border: 1px solid rgba(255,255,255,0.08);
        box-shadow: 0 12px 40px rgba(0,0,0,0.4), inset 0 1px 0 rgba(255,255,255,0.06);
      "
    >
      <div class="flex items-center gap-2">
        <div
          class="rounded-full shrink-0"
          style="width: 8px; height: 8px; background: {profileColor(profileName)};"
        ></div>
        <p class="text-lg font-semibold text-white">{profileName}</p>
      </div>
      <p class="text-[13px] text-white/70">
        {modelId} &middot; {language}
      </p>
    </div>
  {/if}
</div>

<style>
  @keyframes switchEnter {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .switch-enter {
    animation: switchEnter 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  }
</style>
