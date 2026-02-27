<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  interface ModelStatus {
    id: string;
    engine: string;
    name: string;
    displayName: string;
    size: string;
    languages: string[];
    downloadUrl: string;
    isDirectory: boolean;
    downloaded: boolean;
    active: boolean;
  }

  interface EngineGroup {
    engine: string;
    displayName: string;
    description: string;
    models: ModelStatus[];
  }

  let groups: EngineGroup[] = $state([]);
  let selectedModelId = $state("whisper-small");
  let downloadProgress = $state<Record<string, number>>({});
  let downloadError = $state<string | null>(null);
  let downloadingModels = $state<Record<string, boolean>>({});
  let extractingModels = $state<Record<string, boolean>>({});
  let expandedEngines = $state<Record<string, boolean>>({
    whisper: true,
    parakeet: true,
  });
  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    await loadSelectedModel();
    await loadModels();

    const u1 = await listen<{ modelId: string; progress: number }>(
      "model-download-progress",
      (event) => {
        downloadProgress = {
          ...downloadProgress,
          [event.payload.modelId]: event.payload.progress,
        };
      },
    );
    unlisteners.push(u1);

    const u2 = await listen<{ modelId: string }>(
      "model-download-complete",
      async (event) => {
        const mid = event.payload.modelId;
        delete downloadingModels[mid];
        downloadingModels = { ...downloadingModels };
        delete downloadProgress[mid];
        downloadProgress = { ...downloadProgress };
        delete extractingModels[mid];
        extractingModels = { ...extractingModels };
        await loadModels();
      },
    );
    unlisteners.push(u2);

    const u3 = await listen<{ modelId: string; error: string }>(
      "model-download-error",
      (event) => {
        const mid = event.payload.modelId;
        delete downloadingModels[mid];
        downloadingModels = { ...downloadingModels };
        delete downloadProgress[mid];
        downloadProgress = { ...downloadProgress };
        delete extractingModels[mid];
        extractingModels = { ...extractingModels };
        downloadError = event.payload.error;
      },
    );
    unlisteners.push(u3);

    // Listen for extraction events (tar.gz directory models)
    const u4 = await listen<{ modelId: string }>(
      "model-extracting",
      (event) => {
        extractingModels = {
          ...extractingModels,
          [event.payload.modelId]: true,
        };
      },
    );
    unlisteners.push(u4);

    const u5 = await listen<{ modelId: string }>(
      "model-extraction-complete",
      (event) => {
        delete extractingModels[event.payload.modelId];
        extractingModels = { ...extractingModels };
      },
    );
    unlisteners.push(u5);
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
  });

  async function loadModels() {
    try {
      groups = (await invoke("list_models_grouped", {
        activeModelId: selectedModelId,
      })) as EngineGroup[];
    } catch (e) {
      console.error("Failed to load models:", e);
    }
  }

  async function loadSelectedModel() {
    try {
      const settings = (await invoke("get_settings")) as {
        selectedModel: string;
      };
      selectedModelId = settings.selectedModel;
      // Expand the engine group that contains the selected model
      const engine = selectedModelId.split("-")[0];
      expandedEngines = { ...expandedEngines, [engine]: true };
    } catch {
      // Use default
    }
  }

  async function selectModel(modelId: string) {
    selectedModelId = modelId;
    await invoke("update_setting", {
      key: "selectedModel",
      value: modelId,
    });
    await loadModels();
  }

  async function downloadOrLoadModel() {
    const model = getSelectedModel();
    if (!model) return;

    // If already downloaded, just load (select) it — don't re-download
    if (model.downloaded) {
      await selectModel(model.id);
      return;
    }

    // Skip if this specific model is already downloading
    if (downloadingModels[model.id]) return;

    downloadingModels = { ...downloadingModels, [model.id]: true };
    downloadError = null;

    try {
      await invoke("download_model", { modelId: model.id });
    } catch (e) {
      delete downloadingModels[model.id];
      downloadingModels = { ...downloadingModels };
      downloadError = String(e);
    }
  }

  async function deleteModel(modelId: string) {
    try {
      await invoke("delete_model", { modelId });
      await loadModels();
    } catch (e) {
      console.error("Failed to delete model:", e);
    }
  }

  function toggleEngine(engine: string) {
    expandedEngines = {
      ...expandedEngines,
      [engine]: !expandedEngines[engine],
    };
  }

  function getSelectedModel(): ModelStatus | undefined {
    for (const group of groups) {
      const found = group.models.find((m) => m.id === selectedModelId);
      if (found) return found;
    }
    return undefined;
  }

  function canDownload(): boolean {
    const model = getSelectedModel();
    if (!model) return false;
    if (downloadingModels[model.id]) return false;
    if (model.downloaded && model.active) return false;
    return true;
  }

  function buttonTitle(): string {
    const model = getSelectedModel();
    if (!model) return "Select a model";
    if (downloadingModels[model.id]) return "Downloading...";
    if (model.downloaded && model.active) return "Model Ready";
    if (model.downloaded) return `Load ${model.displayName.split(" (")[0]}`;
    return `Download ${model.displayName.split(" (")[0]}`;
  }

  function getProgress(modelId: string): number | undefined {
    return downloadProgress[modelId];
  }

  function isExtracting(modelId: string): boolean {
    return !!extractingModels[modelId];
  }
</script>

<div class="space-y-4">
  {#each groups as group}
    <section>
      <!-- Engine header (clickable to expand/collapse) -->
      <button
        class="w-full flex items-center gap-2 text-left px-3 py-1.5 hover:bg-white/5 rounded-lg transition-colors"
        onclick={() => toggleEngine(group.engine)}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          class="w-3.5 h-3.5 text-white/40 transition-transform {expandedEngines[
            group.engine
          ]
            ? 'rotate-90'
            : ''}"
        >
          <path
            fill-rule="evenodd"
            d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
            clip-rule="evenodd"
          />
        </svg>
        <div>
          <h3
            class="text-xs font-semibold text-white/60 uppercase tracking-wider"
          >
            {group.displayName}
          </h3>
          <p class="text-[10px] text-white/30">{group.description}</p>
        </div>
      </button>

      <!-- Model list (collapsible) -->
      {#if expandedEngines[group.engine]}
        <div class="space-y-0.5 mt-1">
          {#each group.models as model}
            <label
              class="flex items-center gap-3 py-1.5 px-3 rounded-lg hover:bg-white/5 cursor-pointer"
            >
              <input
                type="radio"
                name="model"
                value={model.id}
                checked={selectedModelId === model.id}
                onchange={() => selectModel(model.id)}
                class="w-3.5 h-3.5 accent-blue-500"
              />
              <div class="flex-1 min-w-0">
                <span class="text-[13px]">{model.displayName}</span>
              </div>
              <div class="flex items-center gap-2 shrink-0">
                {#if isExtracting(model.id)}
                  <span class="text-[10px] text-blue-400 animate-pulse"
                    >Extracting...</span
                  >
                {:else if getProgress(model.id) !== undefined}
                  <div
                    class="w-16 h-1.5 bg-white/10 rounded-full overflow-hidden"
                  >
                    <div
                      class="h-full bg-blue-500 rounded-full transition-all"
                      style="width: {(getProgress(model.id) ?? 0) * 100}%"
                    ></div>
                  </div>
                  <span class="text-[10px] text-white/50 w-8 text-right">
                    {Math.round((getProgress(model.id) ?? 0) * 100)}%
                  </span>
                {:else if model.downloaded}
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="#22c55e"
                    class="w-4 h-4"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  <button
                    class="text-xs text-red-400/60 hover:text-red-400 transition-colors"
                    onclick={() => deleteModel(model.id)}
                    title="Delete model"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      viewBox="0 0 16 16"
                      fill="currentColor"
                      class="w-3.5 h-3.5"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M5 3.25V4H2.75a.75.75 0 000 1.5h.3l.815 8.15A1.5 1.5 0 005.357 15h5.286a1.5 1.5 0 001.492-1.35l.815-8.15h.3a.75.75 0 000-1.5H11v-.75A2.25 2.25 0 008.75 1h-1.5A2.25 2.25 0 005 3.25zm2.25-.75a.75.75 0 00-.75.75V4h3v-.75a.75.75 0 00-.75-.75h-1.5zM6.05 6a.75.75 0 01.787.713l.275 5.5a.75.75 0 01-1.498.075l-.275-5.5A.75.75 0 016.05 6zm3.9 0a.75.75 0 01.712.787l-.275 5.5a.75.75 0 01-1.498-.075l.275-5.5a.75.75 0 01.786-.712z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </button>
                {:else if downloadingModels[model.id]}
                  <div
                    class="w-16 h-1.5 bg-white/10 rounded-full overflow-hidden"
                  >
                    <div
                      class="h-full bg-blue-500 rounded-full transition-all"
                      style="width: {(getProgress(model.id) ?? 0) * 100}%"
                    ></div>
                  </div>
                  <span class="text-[10px] text-white/50 w-8 text-right">
                    {Math.round((getProgress(model.id) ?? 0) * 100)}%
                  </span>
                {:else}
                  <span class="text-[10px] text-white/30">Not downloaded</span>
                {/if}
              </div>
            </label>
          {/each}
        </div>
      {/if}
    </section>
  {/each}

  <!-- Error -->
  {#if downloadError}
    <div class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20">
      <p class="text-xs text-red-400 flex items-center gap-1.5">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          class="w-4 h-4"
        >
          <path
            fill-rule="evenodd"
            d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-5a.75.75 0 01.75.75v4.5a.75.75 0 01-1.5 0v-4.5A.75.75 0 0110 5zm0 10a1 1 0 100-2 1 1 0 000 2z"
            clip-rule="evenodd"
          />
        </svg>
        {downloadError}
      </p>
    </div>
  {/if}

  <!-- Download/Load button -->
  <div class="px-3">
    <button
      class="w-full py-2 px-4 rounded-lg text-sm font-medium transition-colors
        {canDownload()
        ? 'bg-blue-500 hover:bg-blue-600 text-white'
        : 'bg-white/5 text-white/30 cursor-not-allowed'}"
      disabled={!canDownload()}
      onclick={downloadOrLoadModel}
    >
      {buttonTitle()}
    </button>
  </div>

  <!-- Recommendation -->
  <p class="px-3 text-xs text-white/40">
    Try Parakeet V3 for speed or Whisper Small for accuracy.
  </p>
</div>
