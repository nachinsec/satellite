import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Version } from "../utils/types";
import { APP_CONSTANTS } from "../constants";

export const useLauncher = () => {
  const [logs, setLogs] = createSignal<string[]>([]);
  const [progress, setProgress] = createSignal(0);
  const [showAll, setShowAll] = createSignal(false);
  const [selectedVersion, setSelectedVersion] = createSignal<string>(APP_CONSTANTS.DEFAULT_VERSION);
  const [showVersionSelector, setShowVersionSelector] = createSignal(false);
  const [versions, setVersions] = createSignal<Version[]>([]);
  const [config, setConfig] = createSignal<any>({});
  const [playerName, setPlayerName] = createSignal(APP_CONSTANTS.DEFAULT_PLAYER_NAME);
  const [activeSection, setActiveSection] = createSignal("Home");

  let logRef: HTMLDivElement | undefined;

  // Tauri command wrappers
  async function startLauncher() {
    await invoke("update_config", { config: { ...config(), player_name: playerName() } });
    await invoke("start_launcher", { version: selectedVersion() });
  }

  async function getConfig() {
    const config: any = await invoke("get_config");
    setConfig(config);
    setPlayerName(config.player_name || APP_CONSTANTS.DEFAULT_PLAYER_NAME);
  }

  async function updateConfig(config: any) {
    setConfig(config);
  }

  async function getVersions() {
    const versions = await invoke("get_versions");
    setVersions(versions as Version[]);
  }

  // Log management
  function getLog(): string {
    const all = logs();
    if (showAll()) return all.join("\n");
    return all.slice(-APP_CONSTANTS.MAX_LOGS).join("\n");
  }

  function toggleShowAll() {
    setShowAll((v) => {
      logRef?.scrollTo({ top: logRef.scrollHeight });
      return !v;
    });
  }

  function handleVersionSelect(version: string) {
    setSelectedVersion(version);
    setShowVersionSelector(false);
  }

  // Initialize on mount
  onMount(() => {
    getVersions();
    getConfig();
  });

  // Event listeners
  listen("log", (event) => {
    setLogs((logs) => [...logs, event.payload as string]);
    logRef?.scrollTo({ top: logRef.scrollHeight });
  });

  listen("progress", (event) => {
    setProgress(event.payload as number);
  });

  return {
    // State
    logs,
    progress,
    showAll,
    selectedVersion,
    showVersionSelector,
    versions,
    config,
    playerName,
    activeSection,
    logRef: (ref: HTMLDivElement) => { logRef = ref; },
    
    // Actions
    startLauncher,
    getConfig,
    updateConfig,
    getVersions,
    getLog,
    toggleShowAll,
    handleVersionSelect,
    
    // Setters
    setPlayerName,
    setActiveSection,
    setShowVersionSelector,
  };
};
