/** Application state phases */
export type AppPhase = "idle" | "recording" | "processing" | "error";

/** Transcription engine types */
export type EngineType = "whisper" | "parakeet" | "moonshine" | "sensevoice";

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

/** A completed transcription */
export interface TranscriptionEntry {
  id: string;
  text: string;
  modelId: string;
  duration: number;
  timestamp: number;
}

/** Audio level event from recording */
export interface AudioLevelEvent {
  rms: number;
  peak: number;
}

/** Permission status for system capabilities */
export type PermissionStatus = "granted" | "denied" | "unknown";

/** All permission states */
export interface Permissions {
  microphone: PermissionStatus;
  accessibility: PermissionStatus;
  inputMonitoring: PermissionStatus;
}

/** Settings stored in tauri-plugin-store */
export interface Settings {
  hotkey: string;
  autoPaste: boolean;
  autoStart: boolean;
  removeFiller: boolean;
  activeModelId: string;
  activeProfileId: string;
  profiles: Profile[];
  switchProfileHotkey: string;
}
