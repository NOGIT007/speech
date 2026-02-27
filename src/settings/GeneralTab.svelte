<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import HotkeyRecorder from "./HotkeyRecorder.svelte";

  interface Settings {
    launchAtLogin: boolean;
    autoPaste: boolean;
    removeFillerWords: boolean;
    recordHotkey: string;
    switchHotkey: string;
    switchHotkeyEnabled: boolean;
  }

  let launchAtLogin = $state(false);
  let autoPaste = $state(true);
  let removeFillerWords = $state(true);
  let recordHotkey = $state("Alt+Space");
  let switchHotkey = $state("Alt+Shift+Space");
  let switchHotkeyEnabled = $state(false);

  onMount(async () => {
    try {
      const settings = (await invoke("get_settings")) as Settings;
      launchAtLogin = settings.launchAtLogin;
      autoPaste = settings.autoPaste;
      removeFillerWords = settings.removeFillerWords;
      recordHotkey = settings.recordHotkey;
      switchHotkey = settings.switchHotkey;
      switchHotkeyEnabled = settings.switchHotkeyEnabled;
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  });

  async function updateSetting(key: string, value: unknown) {
    try {
      await invoke("update_setting", { key, value });
    } catch (e) {
      console.error(`Failed to update ${key}:`, e);
    }
  }

  async function toggleLaunchAtLogin() {
    const prev = launchAtLogin;
    launchAtLogin = !launchAtLogin;
    try {
      await updateSetting("launchAtLogin", launchAtLogin);
    } catch {
      launchAtLogin = prev;
    }
  }

  async function toggleAutoPaste() {
    const prev = autoPaste;
    autoPaste = !autoPaste;
    try {
      await updateSetting("autoPaste", autoPaste);
    } catch {
      autoPaste = prev;
    }
  }

  async function toggleRemoveFillerWords() {
    const prev = removeFillerWords;
    removeFillerWords = !removeFillerWords;
    try {
      await updateSetting("removeFillerWords", removeFillerWords);
    } catch {
      removeFillerWords = prev;
    }
  }

  async function toggleSwitchHotkeyEnabled() {
    const prev = switchHotkeyEnabled;
    switchHotkeyEnabled = !switchHotkeyEnabled;
    try {
      await updateSetting("switchHotkeyEnabled", switchHotkeyEnabled);
    } catch {
      switchHotkeyEnabled = prev;
    }
  }

  function onRecordHotkeyChange(shortcut: string) {
    recordHotkey = shortcut;
    updateSetting("recordHotkey", shortcut);
  }

  function onSwitchHotkeyChange(shortcut: string) {
    switchHotkey = shortcut;
    updateSetting("switchHotkey", shortcut);
  }
</script>

<div class="space-y-6">
  <!-- Preferences section -->
  <section>
    <h3 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">
      Preferences
    </h3>
    <div class="space-y-1">
      <label class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-white/5 cursor-pointer">
        <span class="text-sm">Launch at login</span>
        <button
          class="relative w-9 h-5 rounded-full transition-colors {launchAtLogin ? 'bg-blue-500' : 'bg-white/20'}"
          onclick={toggleLaunchAtLogin}
          role="switch"
          aria-checked={launchAtLogin}
        >
          <span
            class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform
              {launchAtLogin ? 'translate-x-4' : 'translate-x-0'}"
          ></span>
        </button>
      </label>

      <label class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-white/5 cursor-pointer">
        <span class="text-sm">Auto-paste text</span>
        <button
          class="relative w-9 h-5 rounded-full transition-colors {autoPaste ? 'bg-blue-500' : 'bg-white/20'}"
          onclick={toggleAutoPaste}
          role="switch"
          aria-checked={autoPaste}
        >
          <span
            class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform
              {autoPaste ? 'translate-x-4' : 'translate-x-0'}"
          ></span>
        </button>
      </label>

      <label class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-white/5 cursor-pointer">
        <span class="text-sm">Remove filler words</span>
        <button
          class="relative w-9 h-5 rounded-full transition-colors {removeFillerWords ? 'bg-blue-500' : 'bg-white/20'}"
          onclick={toggleRemoveFillerWords}
          role="switch"
          aria-checked={removeFillerWords}
        >
          <span
            class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform
              {removeFillerWords ? 'translate-x-4' : 'translate-x-0'}"
          ></span>
        </button>
      </label>
    </div>
  </section>

  <!-- Dictation Hotkey section -->
  <section>
    <h3 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">
      Dictation Hotkey
    </h3>
    <div class="px-3">
      <HotkeyRecorder
        label="Dictation shortcut"
        shortcut={recordHotkey}
        defaultShortcut="Alt+Space"
        onchange={onRecordHotkeyChange}
      />
      <p class="text-xs text-white/40 mt-2">
        Hold to record, release to transcribe
      </p>
    </div>
  </section>

  <!-- Profile Switching section -->
  <section>
    <h3 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">
      Profile Switching
    </h3>
    <div class="px-3">
      <label class="flex items-center justify-between py-2 cursor-pointer">
        <span class="text-sm">Enable quick switch</span>
        <button
          class="relative w-9 h-5 rounded-full transition-colors {switchHotkeyEnabled ? 'bg-blue-500' : 'bg-white/20'}"
          onclick={toggleSwitchHotkeyEnabled}
          role="switch"
          aria-checked={switchHotkeyEnabled}
        >
          <span
            class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform
              {switchHotkeyEnabled ? 'translate-x-4' : 'translate-x-0'}"
          ></span>
        </button>
      </label>

      {#if switchHotkeyEnabled}
        <div class="mt-2">
          <HotkeyRecorder
            label="Switch shortcut"
            shortcut={switchHotkey}
            defaultShortcut="Alt+Shift+Space"
            onchange={onSwitchHotkeyChange}
          />

          {#if switchHotkey === recordHotkey}
            <p class="text-xs text-orange-400 mt-2 flex items-center gap-1">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-3.5 h-3.5">
                <path fill-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
              </svg>
              Conflicts with dictation hotkey
            </p>
          {/if}
        </div>

        <p class="text-xs text-white/40 mt-2">
          Tap to cycle between profiles
        </p>
      {/if}
    </div>
  </section>
</div>
