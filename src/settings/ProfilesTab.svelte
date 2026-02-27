<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ProfileCard from "./ProfileCard.svelte";

  interface ModelProfile {
    id: string;
    name: string;
    modelId: string;
    language: string;
  }

  interface ModelOption {
    id: string;
    displayName: string;
  }

  interface LanguageOption {
    code: string;
    name: string;
  }

  let profiles = $state<ModelProfile[]>([]);
  let activeIndex = $state(0);
  let models = $state<ModelOption[]>([]);
  let languages = $state<LanguageOption[]>([]);

  onMount(async () => {
    await loadProfiles();
    await loadModels();
    await loadLanguages();
    // Migrate on first launch
    try {
      await invoke("migrate_profiles");
      await loadProfiles();
    } catch {
      // ignore
    }
  });

  async function loadProfiles() {
    try {
      profiles = (await invoke("list_profiles")) as ModelProfile[];
      activeIndex = (await invoke("get_active_profile_index")) as number;
    } catch (e) {
      console.error("Failed to load profiles:", e);
    }
  }

  async function loadModels() {
    try {
      const result = (await invoke("list_models", {
        activeModelId: null,
      })) as { id: string; displayName: string; downloaded: boolean }[];
      models = result
        .filter((m) => m.downloaded)
        .map((m) => ({ id: m.id, displayName: m.displayName }));
    } catch (e) {
      console.error("Failed to load models:", e);
    }
  }

  async function loadLanguages() {
    try {
      languages = (await invoke("get_supported_languages")) as LanguageOption[];
    } catch (e) {
      console.error("Failed to load languages:", e);
    }
  }

  async function addProfile() {
    try {
      await invoke("create_profile", {
        name: "New",
        modelId: "whisper-small",
        language: "en",
      });
      await loadProfiles();
    } catch (e) {
      console.error("Failed to create profile:", e);
    }
  }

  async function deleteProfile(id: string) {
    try {
      await invoke("delete_profile", { id });
      await loadProfiles();
    } catch (e) {
      console.error("Failed to delete profile:", e);
    }
  }

  async function activateProfile(index: number) {
    try {
      await invoke("set_active_profile", { index });
      activeIndex = index;
    } catch (e) {
      console.error("Failed to activate profile:", e);
    }
  }
</script>

<div class="space-y-6">
  <!-- Profiles section -->
  <section>
    <h3
      class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3"
    >
      Profiles
    </h3>
    <div class="space-y-2">
      {#each profiles as profile, i (profile.id)}
        <ProfileCard
          {profile}
          index={i}
          isActive={i === activeIndex}
          canDelete={profiles.length > 1}
          {models}
          {languages}
          onactivate={() => activateProfile(i)}
          ondelete={() => deleteProfile(profile.id)}
          onupdate={loadProfiles}
        />
      {/each}
    </div>

    <button
      class="mt-3 flex items-center gap-1.5 text-xs text-blue-400 hover:text-blue-300 transition-colors px-3"
      onclick={addProfile}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 20 20"
        fill="currentColor"
        class="w-4 h-4"
      >
        <path
          d="M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z"
        />
      </svg>
      Add Profile
    </button>

    <p class="text-xs text-white/40 mt-2 px-3">
      Preset model + language combinations. Switch between them via hotkey.
    </p>
  </section>
</div>
