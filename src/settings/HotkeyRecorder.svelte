<script lang="ts">
  interface Props {
    label: string;
    shortcut: string;
    defaultShortcut: string;
    onchange: (shortcut: string) => void;
  }

  let { label, shortcut, defaultShortcut, onchange }: Props = $props();

  let isRecording = $state(false);

  function startRecording() {
    isRecording = true;
    // Listen for keydown events to capture the shortcut
    window.addEventListener("keydown", handleKeyDown);
  }

  function stopRecording() {
    isRecording = false;
    window.removeEventListener("keydown", handleKeyDown);
  }

  function handleKeyDown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    // Escape cancels recording
    if (event.key === "Escape") {
      stopRecording();
      return;
    }

    // Don't capture bare modifier keys
    const modifierKeys = [
      "Shift",
      "Control",
      "Alt",
      "Meta",
      "CapsLock",
    ];
    if (modifierKeys.includes(event.key)) {
      return;
    }

    // Require at least one modifier (matching HotkeyRecorderView.swift:47-49)
    if (!event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
      return;
    }

    // Build shortcut string in Tauri format
    const parts: string[] = [];
    if (event.ctrlKey) parts.push("Ctrl");
    if (event.altKey) parts.push("Alt");
    if (event.shiftKey) parts.push("Shift");
    if (event.metaKey) parts.push("Super");

    // Map key to Tauri key name
    const keyName = mapKeyToTauriName(event.key, event.code);
    parts.push(keyName);

    const newShortcut = parts.join("+");
    onchange(newShortcut);
    stopRecording();
  }

  function mapKeyToTauriName(key: string, code: string): string {
    // Map common keys to Tauri's expected format
    const keyMap: Record<string, string> = {
      " ": "Space",
      ArrowUp: "Up",
      ArrowDown: "Down",
      ArrowLeft: "Left",
      ArrowRight: "Right",
      Enter: "Return",
      Backspace: "Backspace",
      Tab: "Tab",
      Delete: "Delete",
    };

    if (keyMap[key]) return keyMap[key];

    // For letter/number keys, use uppercase
    if (key.length === 1) return key.toUpperCase();

    // Function keys
    if (/^F\d+$/.test(key)) return key;

    return key;
  }

  function resetToDefault() {
    onchange(defaultShortcut);
  }

  function formatForDisplay(s: string): string {
    return s
      .replace("Alt", "\u2325")
      .replace("Ctrl", "\u2303")
      .replace("Shift", "\u21E7")
      .replace("Super", "\u2318")
      .replace("Space", "\u2423")
      .replace(/\+/g, "");
  }
</script>

<div class="flex items-center justify-between">
  <span class="text-sm">{label}</span>

  <div class="flex items-center gap-2">
    <button
      class="px-3 py-1.5 text-sm rounded-md border transition-colors min-w-[100px] text-center
        {isRecording
        ? 'border-blue-500 text-blue-400 bg-blue-500/10'
        : 'border-white/20 text-white/80 bg-white/5 hover:bg-white/10'}"
      onclick={() => (isRecording ? stopRecording() : startRecording())}
    >
      {isRecording ? "Press shortcut..." : formatForDisplay(shortcut)}
    </button>

    {#if shortcut !== defaultShortcut}
      <button
        class="text-white/40 hover:text-white/60 transition-colors"
        onclick={resetToDefault}
        title="Reset to default ({formatForDisplay(defaultShortcut)})"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-4 h-4">
          <path fill-rule="evenodd" d="M15.312 11.424a5.5 5.5 0 01-9.201 2.466l-.312-.311h2.433a.75.75 0 000-1.5H4.644a.75.75 0 00-.75.75v3.588a.75.75 0 001.5 0v-2.434l.312.311a7 7 0 0011.712-3.138.75.75 0 00-1.449-.39zm-10.624-6.27a7 7 0 0111.712 3.138.75.75 0 01-1.449.39 5.5 5.5 0 00-9.201-2.466l-.312.311h2.433a.75.75 0 010 1.5H4.644a.75.75 0 01-.75-.75V3.589a.75.75 0 011.5 0v2.434l.312-.311z" clip-rule="evenodd" />
        </svg>
      </button>
    {/if}
  </div>
</div>
