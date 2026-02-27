/** Application state phases (matches Rust AppPhase enum) */
export type AppPhase = "idle" | "recording" | "processing";

/** Transcription engine types */
export type EngineType = "whisper" | "parakeet";

/** A model available for transcription */
export interface Model {
  id: string;
  engine: EngineType;
  name: string;
  displayName: string;
  size: string;
  languages: string[];
  downloaded: boolean;
  active: boolean;
  downloadProgress?: number;
}

/** A user-defined model profile */
export interface Profile {
  id: string;
  name: string;
  modelId: string;
  language: string;
}

/** A transcription history item (matches Rust TranscriptionItem) */
export interface TranscriptionItem {
  id: string;
  text: string;
  timestamp: string;
  preview: string;
}

/** Audio level event from recording */
export interface AudioLevelEvent {
  rms: number;
  peak: number;
}

/** Permission status for system capabilities */
export interface PermissionStatus {
  microphone: boolean;
  accessibility: boolean;
  inputMonitoring: boolean;
}

/** Settings stored in tauri-plugin-store (matches Rust AppSettings) */
export interface Settings {
  launchAtLogin: boolean;
  autoPaste: boolean;
  removeFillerWords: boolean;
  recordHotkey: string;
  switchHotkey: string;
  switchHotkeyEnabled: boolean;
  selectedLanguage: string;
  selectedModel: string;
  activeProfileIndex: number;
}
