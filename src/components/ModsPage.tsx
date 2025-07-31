import { createSignal, createEffect, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ModInfo, ModSearchResult, ModLoader } from "../types/mods";
import { CSS_CLASSES } from "../constants";
import toast from "solid-toast";

interface ModsPageProps {
  gameDirectory: string;
  minecraftVersion: string;
}

export function ModsPage(props: ModsPageProps) {
  const [installedMods, setInstalledMods] = createSignal<ModInfo[]>([]);
  const [searchResults, setSearchResults] = createSignal<ModSearchResult[]>([]);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [isSearching, setIsSearching] = createSignal(false);
  const [activeTab, setActiveTab] = createSignal<"installed" | "browse">("installed");
  const [loading, setLoading] = createSignal(false);

  // Load installed mods on component mount
  createEffect(() => {
    if (props.gameDirectory) {
      loadInstalledMods();
    }
  });

  const loadInstalledMods = async () => {
    try {
      setLoading(true);
      const mods = await invoke<ModInfo[]>("get_installed_mods", {
        gameDirectory: props.gameDirectory,
      });
      setInstalledMods(mods);
    } catch (error) {
      console.error("Failed to load mods:", error);
      toast.error("Failed to load installed mods");
    } finally {
      setLoading(false);
    }
  };

  const toggleMod = async (modId: string, enabled: boolean) => {
    try {
      await invoke("toggle_mod", {
        gameDirectory: props.gameDirectory,
        modId,
        enabled,
      });
      
      // Update local state
      setInstalledMods(mods => 
        mods.map(mod => 
          mod.id === modId ? { ...mod, enabled } : mod
        )
      );
      
      toast.success(`Mod ${enabled ? "enabled" : "disabled"}`);
    } catch (error) {
      console.error("Failed to toggle mod:", error);
      toast.error("Failed to toggle mod");
    }
  };

  const deleteMod = async (modId: string) => {
    if (!confirm("Are you sure you want to delete this mod?")) return;
    
    try {
      await invoke("delete_mod", {
        gameDirectory: props.gameDirectory,
        modId,
      });
      
      setInstalledMods(mods => mods.filter(mod => mod.id !== modId));
      toast.success("Mod deleted successfully");
    } catch (error) {
      console.error("Failed to delete mod:", error);
      toast.error("Failed to delete mod");
    }
  };

  const installModFromFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: "Mod Files",
          extensions: ["jar"]
        }]
      });

      if (selected && typeof selected === "string") {
        const modInfo = await invoke<ModInfo>("install_mod_from_file", {
          gameDirectory: props.gameDirectory,
          filePath: selected,
        });
        
        setInstalledMods(mods => [...mods, modInfo]);
        toast.success("Mod installed successfully");
      }
    } catch (error) {
      console.error("Failed to install mod:", error);
      toast.error("Failed to install mod");
    }
  };

  const searchMods = async () => {
    if (!searchQuery().trim()) return;
    
    try {
      setIsSearching(true);
      const results = await invoke<ModSearchResult[]>("search_mods_online", {
        query: searchQuery(),
        minecraftVersion: props.minecraftVersion,
        modLoader: ModLoader.Fabric, // Default to Fabric for now
        limit: 20,
      });
      setSearchResults(results);
    } catch (error) {
      console.error("Failed to search mods:", error);
      toast.error("Failed to search mods");
    } finally {
      setIsSearching(false);
    }
  };

  const formatFileSize = (bytes: number): string => {
    const sizes = ['B', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  return (
    <div class="flex flex-col h-full p-6 w-full">
      <div class="mb-6">
        <h1 class={CSS_CLASSES.TEXT.TITLE}>Mod Management</h1>
        <p class="text-gray-600 dark:text-gray-400 mt-2">
          Manage your Minecraft mods for version {props.minecraftVersion}
        </p>
      </div>

      {/* Tab Navigation */}
      <div class="flex mb-6 border-b border-gray-200 dark:border-gray-700">
        <button
          class={`px-4 py-2 font-semibold transition-colors ${
            activeTab() === "installed"
              ? "text-green-600 border-b-2 border-green-600"
              : "text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
          }`}
          onClick={() => setActiveTab("installed")}
        >
          Installed Mods ({installedMods().length})
        </button>
        <button
          class={`px-4 py-2 font-semibold transition-colors ${
            activeTab() === "browse"
              ? "text-green-600 border-b-2 border-green-600"
              : "text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
          }`}
          onClick={() => setActiveTab("browse")}
        >
          Browse Mods
        </button>
      </div>

      <Show when={activeTab() === "installed"}>
        <div class="flex-1">
          <div class="flex justify-between items-center mb-4 gap-4">
            <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-200">
              Installed Mods
            </h2>
          </div>

          <Show
            when={!loading() && installedMods().length > 0}
            fallback={
              <div class="text-center py-12">
                <div class="text-6xl mb-4">🧩</div>
                <h3 class="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-2">
                  No mods installed
                </h3>
                <p class="text-gray-600 dark:text-gray-400 mb-4">
                  Install your first mod to get started
                </p>
                <div class="flex gap-2 justify-center">
                <button
                  class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded transition mt-4"
                  onClick={installModFromFile}
                >
                  Install Mod
                </button>
                <button
                  class={CSS_CLASSES.BUTTON.SECONDARY}
                  onClick={loadInstalledMods}
                  disabled={loading()}
                >
                  {loading() ? "Refreshing..." : "Refresh"}
                </button>
                </div>
              </div>
            }
          >
            <div class="grid gap-4">
              <For each={installedMods()}>
                {(mod) => (
                  <div class="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700 shadow-sm">
                    <div class="flex items-start justify-between">
                      <div class="flex-1">
                        <div class="flex items-center gap-3 mb-2">
                          <h3 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                            {mod.name}
                          </h3>
                          <span class={`px-2 py-1 rounded text-xs font-medium ${
                            mod.enabled 
                              ? "bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-200"
                              : "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300"
                          }`}>
                            {mod.enabled ? "Enabled" : "Disabled"}
                          </span>
                          <span class="px-2 py-1 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-800 dark:text-blue-200">
                            {mod.mod_loader}
                          </span>
                        </div>
                        
                        <div class="text-sm text-gray-600 dark:text-gray-400 space-y-1">
                          <div>Version: {mod.version}</div>
                          <Show when={mod.author}>
                            <div>Author: {mod.author}</div>
                          </Show>
                          <div>File: {mod.file_name} ({formatFileSize(mod.file_size)})</div>
                          <Show when={mod.description}>
                            <div class="mt-2">{mod.description}</div>
                          </Show>
                        </div>
                      </div>
                      
                      <div class="flex gap-2 ml-4">
                        <button
                          class={`px-3 py-1 rounded text-sm font-medium transition-colors ${
                            mod.enabled
                              ? "bg-yellow-100 text-yellow-800 hover:bg-yellow-200 dark:bg-yellow-800 dark:text-yellow-200"
                              : "bg-green-100 text-green-800 hover:bg-green-200 dark:bg-green-800 dark:text-green-200"
                          }`}
                          onClick={() => toggleMod(mod.id, !mod.enabled)}
                        >
                          {mod.enabled ? "Disable" : "Enable"}
                        </button>
                        <button
                          class="px-3 py-1 rounded text-sm font-medium bg-red-100 text-red-800 hover:bg-red-200 dark:bg-red-800 dark:text-red-200 transition-colors"
                          onClick={() => deleteMod(mod.id)}
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={activeTab() === "browse"}>
        <div class="flex-1">
          <div class="mb-6">
            <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-4">
              Browse Mods
            </h2>
            <div class="flex gap-2">
              <input
                type="text"
                placeholder="Cobblemon..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                onKeyPress={(e) => e.key === "Enter" && searchMods()}
                class="w-[80%] px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-2 focus:ring-green-500"
              />
              <button
                class={CSS_CLASSES.BUTTON.PRIMARY}
                onClick={searchMods}
                disabled={isSearching()}
              >
                {isSearching() ? "Searching..." : "Search"}
              </button>
            </div>
          </div>

          <Show
            when={searchResults().length > 0}
            fallback={
              <div class="text-center py-12">
                <div class="text-6xl mb-4">🔍</div>
                <h3 class="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-2">
                  Search for mods
                </h3>
                <p class="text-gray-600 dark:text-gray-400">
                  Use the search bar above to find mods from Modrinth
                </p>
              </div>
            }
          >
            <div class="grid gap-4 overflow-y-auto h-[calc(100vh-200px)] scrollbar-hide">
              <For each={searchResults()}>
                {(mod) => (
                  <div class="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700 shadow-sm">
                    <div class="flex items-start gap-4">
                      <Show when={mod.icon_url}>
                        <img
                          src={mod.icon_url}
                          alt={mod.name}
                          class="w-16 h-16 rounded-lg object-cover"
                        />
                      </Show>
                      
                      <div class="flex-1">
                        <div class="flex items-center gap-3 mb-2">
                          <h3 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
                            {mod.name}
                          </h3>
                          <span class="text-sm text-gray-600 dark:text-gray-400">
                            by {mod.author}
                          </span>
                        </div>
                        
                        <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">
                          {mod.description}
                        </p>
                        
                        <div class="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400">
                          <span>{mod.downloads.toLocaleString()} downloads</span>
                          <span>Supports: {mod.minecraft_versions.slice(0, 3).join(", ")}</span>
                          <span>Loaders: {mod.mod_loaders.join(", ")}</span>
                        </div>
                      </div>
                      
                      <button
                        class={CSS_CLASSES.BUTTON.PRIMARY}
                        onClick={() => toast("Online mod installation coming soon!")}
                      >
                        Install
                      </button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}
