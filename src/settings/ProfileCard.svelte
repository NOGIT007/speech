<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface ModelOption {
    id: string;
    displayName: string;
  }

  interface LanguageOption {
    code: string;
    name: string;
  }

  interface Profile {
    id: string;
    name: string;
    modelId: string;
    language: string;
  }

  interface Props {
    profile: Profile;
    index: number;
    isActive: boolean;
    canDelete: boolean;
    models: ModelOption[];
    languages: LanguageOption[];
    onactivate: () => void;
    ondelete: () => void;
    onupdate: () => void;
  }

  let {
    profile,
    index,
    isActive,
    canDelete,
    models,
    languages,
    onactivate,
    ondelete,
    onupdate,
  }: Props = $props();

  async function updateName(e: Event) {
    const input = e.target as HTMLInputElement;
    try {
      await invoke("update_profile", { id: profile.id, name: input.value });
      onupdate();
    } catch (err) {
      console.error("Failed to update profile name:", err);
    }
  }

  async function updateModel(e: Event) {
    const select = e.target as HTMLSelectElement;
    try {
      await invoke("update_profile", { id: profile.id, modelId: select.value });
      onupdate();
    } catch (err) {
      console.error("Failed to update profile model:", err);
    }
  }

  async function updateLanguage(e: Event) {
    const select = e.target as HTMLSelectElement;
    try {
      await invoke("update_profile", {
        id: profile.id,
        language: select.value,
      });
      onupdate();
    } catch (err) {
      console.error("Failed to update profile language:", err);
    }
  }
</script>

<div class="space-y-2 p-3 rounded-lg bg-white/5">
  <!-- Header: active indicator + name + delete -->
  <div class="flex items-center gap-2">
    <button
      class="shrink-0"
      onclick={onactivate}
      title={isActive ? "Active profile" : "Set as active"}
    >
      {#if isActive}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="#22c55e"
          class="w-5 h-5"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z"
            clip-rule="evenodd"
          />
        </svg>
      {:else}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="w-5 h-5 text-white/30"
        >
          <circle cx="12" cy="12" r="9" />
        </svg>
      {/if}
    </button>

    <input
      type="text"
      value={profile.name}
      onchange={updateName}
      onfocus={(e) => (e.target as HTMLInputElement).select()}
      class="flex-1 bg-transparent border-b border-white/10 focus:border-blue-500 text-sm text-white outline-none px-1 py-0.5"
      placeholder="Profile name"
    />

    {#if canDelete}
      <button
        class="text-white/20 hover:text-red-400 transition-colors shrink-0"
        onclick={ondelete}
        title="Delete profile"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          class="w-4 h-4"
        >
          <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
        </svg>
      </button>
    {/if}
  </div>

  <!-- Model picker -->
  <div class="flex items-center justify-between">
    <span class="text-xs text-white/40">Model</span>
    <select
      value={profile.modelId}
      onchange={updateModel}
      class="bg-white/5 border border-white/10 rounded text-xs text-white/80 px-2 py-1 outline-none focus:border-blue-500 max-w-[160px]"
    >
      {#each models as model}
        <option value={model.id}>{model.displayName}</option>
      {/each}
    </select>
  </div>

  <!-- Language picker -->
  <div class="flex items-center justify-between">
    <span class="text-xs text-white/40">Language</span>
    <select
      value={profile.language}
      onchange={updateLanguage}
      class="bg-white/5 border border-white/10 rounded text-xs text-white/80 px-2 py-1 outline-none focus:border-blue-500 max-w-[160px]"
    >
      {#each languages as lang}
        <option value={lang.code}>{lang.name}</option>
      {/each}
    </select>
  </div>
</div>
