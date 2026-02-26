<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  interface PermissionStatus {
    microphone: boolean;
    accessibility: boolean;
    inputMonitoring: boolean;
  }

  let permissions = $state<PermissionStatus>({
    microphone: false,
    accessibility: false,
    inputMonitoring: false,
  });
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await refreshPermissions();
    // Poll every 1 second (matching SettingsView.swift:311)
    pollInterval = setInterval(refreshPermissions, 1000);
  });

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
  });

  async function refreshPermissions() {
    try {
      permissions = (await invoke("check_permissions")) as PermissionStatus;
    } catch (e) {
      console.error("Failed to check permissions:", e);
    }
  }

  async function openSettings(type: string) {
    try {
      await invoke("open_permission_settings", { permissionType: type });
    } catch (e) {
      console.error("Failed to open permission settings:", e);
    }
  }

  async function resetPermissions() {
    try {
      await invoke("reset_permissions");
    } catch (e) {
      console.error("Failed to reset permissions:", e);
    }
  }

  const permissionRows = [
    {
      key: "microphone" as const,
      title: "Microphone",
      description: "Required for voice recording",
    },
    {
      key: "accessibility" as const,
      title: "Accessibility",
      description: "Required for text injection",
    },
    {
      key: "inputMonitoring" as const,
      title: "Input Monitoring",
      description: "Required for keyboard simulation",
    },
  ];

  function allGranted(): boolean {
    return permissions.microphone && permissions.accessibility && permissions.inputMonitoring;
  }
</script>

<div class="space-y-6">
  <!-- Required Permissions section -->
  <section>
    <h3 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">
      Required Permissions
    </h3>
    <div class="space-y-1">
      {#each permissionRows as row}
        <div class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-white/5">
          <div>
            <p class="text-sm">{row.title}</p>
            <p class="text-xs text-white/40">{row.description}</p>
          </div>
          <div>
            {#if permissions[row.key]}
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
              <button
                class="px-3 py-1 text-xs font-medium rounded-md bg-blue-500 hover:bg-blue-600 text-white transition-colors"
                onclick={() => openSettings(row.key)}
              >
                Grant
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Status section -->
  <section class="px-3">
    {#if allGranted()}
      <p class="text-xs text-green-400 flex items-center gap-1.5">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-4 h-4">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z" clip-rule="evenodd" />
        </svg>
        All permissions granted. Speech is ready to use.
      </p>
    {:else}
      <p class="text-xs text-white/40">
        Grant all permissions for Speech to work correctly.
      </p>
    {/if}
  </section>

  <!-- Reset section -->
  {#if !permissions.accessibility || !permissions.inputMonitoring}
    <section class="px-3">
      <button
        class="w-full py-2 px-4 rounded-lg text-sm font-medium bg-red-500/10 text-red-400 hover:bg-red-500/20 border border-red-500/20 transition-colors"
        onclick={resetPermissions}
      >
        Reset &amp; Re-grant Permissions
      </button>
      <p class="text-xs text-white/40 mt-2">
        Use this if permissions appear stuck. Clears stale entries and restarts the app so you can re-grant them.
      </p>
    </section>
  {/if}
</div>
