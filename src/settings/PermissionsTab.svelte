<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import {
    requestAccessibilityPermission,
    requestMicrophonePermission,
    requestInputMonitoringPermission,
  } from "tauri-plugin-macos-permissions-api";

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
  let granting = $state(false);

  function startPolling() {
    stopPolling();
    pollInterval = setInterval(refreshPermissions, 3000);
  }

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  function handleVisibility() {
    if (document.hidden) {
      stopPolling();
    } else {
      refreshPermissions();
      startPolling();
    }
  }

  onMount(async () => {
    await refreshPermissions();
    startPolling();
    document.addEventListener("visibilitychange", handleVisibility);
  });

  onDestroy(() => {
    stopPolling();
    document.removeEventListener("visibilitychange", handleVisibility);
  });

  async function refreshPermissions() {
    try {
      permissions = (await invoke("check_permissions")) as PermissionStatus;
    } catch (e) {
      console.error("Failed to check permissions:", e);
    }
  }

  async function grantAll() {
    granting = true;
    try {
      if (!permissions.microphone) await requestMicrophonePermission();
      if (!permissions.accessibility) await requestAccessibilityPermission();
      if (!permissions.inputMonitoring) await requestInputMonitoringPermission();
    } catch (e) {
      console.error("Failed to request permissions:", e);
    }
    granting = false;
  }

  async function openSettings(type: string) {
    try {
      if (type === "microphone") await requestMicrophonePermission();
      else if (type === "accessibility") await requestAccessibilityPermission();
      else if (type === "inputMonitoring") await requestInputMonitoringPermission();
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

  async function relaunchApp() {
    try {
      await invoke("relaunch_app");
    } catch (e) {
      console.error("Failed to relaunch:", e);
    }
  }

  function grantedCount(): number {
    return (
      (permissions.microphone ? 1 : 0) +
      (permissions.accessibility ? 1 : 0) +
      (permissions.inputMonitoring ? 1 : 0)
    );
  }

  function allGranted(): boolean {
    return grantedCount() === 3;
  }

  const permissionRows = [
    {
      key: "microphone" as const,
      title: "Microphone",
      description: "Voice recording",
    },
    {
      key: "accessibility" as const,
      title: "Accessibility",
      description: "Text injection",
    },
    {
      key: "inputMonitoring" as const,
      title: "Input Monitoring",
      description: "Keyboard simulation",
    },
  ];
</script>

<div class="space-y-5">
  <!-- Status indicators -->
  <section>
    <h3
      class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3"
    >
      Permissions ({grantedCount()}/3)
    </h3>
    <div class="space-y-1">
      {#each permissionRows as row}
        <div
          class="flex items-center justify-between py-2 px-3 rounded-lg"
        >
          <div class="flex items-center gap-2.5">
            {#if permissions[row.key]}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="#22c55e"
                class="w-4 h-4 flex-shrink-0"
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
                viewBox="0 0 20 20"
                fill="#f59e0b"
                class="w-4 h-4 flex-shrink-0"
              >
                <path
                  fill-rule="evenodd"
                  d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-5a.75.75 0 01.75.75v4.5a.75.75 0 01-1.5 0v-4.5A.75.75 0 0110 5zm0 10a1 1 0 100-2 1 1 0 000 2z"
                  clip-rule="evenodd"
                />
              </svg>
            {/if}
            <div>
              <p class="text-sm">{row.title}</p>
              <p class="text-xs text-white/40">{row.description}</p>
            </div>
          </div>
          {#if !permissions[row.key]}
            <button
              class="text-xs text-blue-400 hover:text-blue-300 transition-colors"
              onclick={() => openSettings(row.key)}
            >
              Open
            </button>
          {/if}
        </div>
      {/each}
    </div>
  </section>

  <!-- Grant all / Status section -->
  {#if allGranted()}
    <section class="px-3">
      <p class="text-xs text-green-400 flex items-center gap-1.5">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          class="w-4 h-4"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z"
            clip-rule="evenodd"
          />
        </svg>
        All permissions granted. Speech is ready to use.
      </p>
    </section>
  {:else}
    <section class="px-3 space-y-3">
      <button
        class="w-full py-2.5 px-4 rounded-lg text-sm font-medium bg-blue-500 hover:bg-blue-600 text-white transition-colors disabled:opacity-50"
        onclick={grantAll}
        disabled={granting}
      >
        {granting ? "Opening..." : "Grant All Permissions"}
      </button>

      <div class="text-xs text-white/40 space-y-1.5">
        <p>
          This will open System Settings. Enable Speech under both
          <strong class="text-white/60">Accessibility</strong> and
          <strong class="text-white/60">Input Monitoring</strong>.
        </p>
        <p>
          Both are in Privacy & Security — you only need to enter your password
          once.
        </p>
      </div>
    </section>

    <!-- Relaunch hint when some permissions granted but not all -->
    {#if grantedCount() > 0 && !allGranted()}
      <section class="px-3 space-y-2">
        <p class="text-xs text-amber-400">
          Already toggled permissions ON in System Settings? macOS sometimes
          requires a relaunch to detect changes:
        </p>
        <button
          class="w-full py-2 px-4 rounded-lg text-sm font-medium bg-white/5 hover:bg-white/10 text-white/80 border border-white/10 transition-colors"
          onclick={relaunchApp}
        >
          Relaunch Speech
        </button>
      </section>
    {/if}
  {/if}

  <!-- Reset section (last resort) -->
  {#if !allGranted()}
    <section class="px-3 pt-2">
      <button
        class="text-xs text-white/30 hover:text-red-400 transition-colors"
        onclick={resetPermissions}
      >
        Reset &amp; re-grant all permissions
      </button>
      <p class="text-xs text-white/20 mt-1">
        Use if permissions appear stuck after multiple attempts.
      </p>
    </section>
  {/if}
</div>
