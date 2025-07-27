import { createSignal } from "solid-js";
import toast from "solid-toast";
import { open } from "@tauri-apps/plugin-dialog";
import Folder from "../icons/Folder";
export const ConfigPage = (props: { config: any; playerName: string; setPlayerName: (name: string) => void; onSave: (cfg: any) => void }) => {
    const [localName, setLocalName] = createSignal(props.playerName);
    const [localConfig, setLocalConfig] = createSignal({ ...props.config });

    const isDisabled = () => {
        if (localConfig().max_memory < localConfig().min_memory) return true;
        if (localConfig().java_executable === "") return true;
        if (localConfig().game_directory === "") return true;
        if (localName() === "") return true;
        return false;
    }
    return (
      <div class="w-full max-w-lg h-[500px] bg-white dark:bg-gray-900 rounded-xl shadow-lg p-8 border border-gray-200 dark:border-gray-700 animate-fade-in">
        <h2 class="text-2xl font-bold mb-6 text-gray-800 dark:text-gray-100">Launcher Settings</h2>
        <div class="mb-4">
          <label class="block text-sm font-semibold mb-1 text-gray-700 dark:text-gray-200">Player Name</label>
          <input
            class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100"
            value={localName()}
            onInput={e => setLocalName((e.target as HTMLInputElement).value)}
          />
        </div>
        <div class="mb-4">
          <label class="block text-sm font-semibold mb-1 text-gray-700 dark:text-gray-200">Game Directory</label>
          <div class="flex gap-2 items-center">
          <input
            class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100"
            value={localConfig().game_directory || ""}
            onInput={e => setLocalConfig({ ...localConfig(), game_directory: (e.target as HTMLInputElement).value })}
          />
          <button
            class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded transition"
            onClick={async () => {
              const folder = await open({
                directory: true,
                defaultPath: localConfig().game_directory || "./",
              });
              if (folder) {
                setLocalConfig({ ...localConfig(), game_directory: folder + '\\.minecraft' });
              }
            }}
          >
            <Folder />
          </button>
          </div>
        </div>
        <div class="mb-4 flex gap-4">
          <div class="flex-1">
            <label class="block text-sm font-semibold mb-1 text-gray-700 dark:text-gray-200">Min Memory (MB)</label>
            <input
              type="number"
              min={512}
              class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100"
              value={localConfig().min_memory || 1024}
              onInput={e => setLocalConfig({ ...localConfig(), min_memory: Number((e.target as HTMLInputElement).value) })}
            />
          </div>
          <div class="flex-1">
            <label class="block text-sm font-semibold mb-1 text-gray-700 dark:text-gray-200">Max Memory (MB)</label>
            <input
              type="number"
              min={512}
              class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100"
              value={localConfig().max_memory || 4096}
              onInput={e => setLocalConfig({ ...localConfig(), max_memory: Number((e.target as HTMLInputElement).value) })}
            />
          </div>
        </div>
        <div class="mb-4">
          <label class="block text-sm font-semibold mb-1 text-gray-700 dark:text-gray-200">JVM Arguments</label>
          <input
            class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100"
            value={(localConfig().jvm_args || []).join(" ")}
            onInput={e => setLocalConfig({ ...localConfig(), jvm_args: (e.target as HTMLInputElement).value.split(" ") })}
            placeholder="-XX:+UnlockExperimentalVMOptions -XX:+UseG1GC"
          />
        </div>
        <div class="flex justify-end gap-3 mt-8">
          <button
            class="px-6 py-2 rounded-lg bg-gray-300 hover:bg-gray-400 text-gray-800 font-bold transition"
            onClick={() => {
              setLocalName(props.playerName); setLocalConfig({ ...props.config });
            }}
          >
            Cancel
          </button>
          <button
            disabled={isDisabled()}
            class="px-6 py-2 rounded-lg bg-green-500 hover:bg-green-600 text-white font-bold shadow transition"
            onClick={() => {
              props.setPlayerName(localName());
              props.onSave({ ...localConfig(), player_name: localName() });
              toast("Settings saved successfully!");
            }}
          >
            Save
          </button>
        </div>
      </div>
    );
  }