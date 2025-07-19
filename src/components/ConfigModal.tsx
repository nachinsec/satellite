import { createSignal } from "solid-js";
import Folder from "../icons/Folder";
interface ConfigModalProps {
  config: any;
  onClose: () => void;
  onSave: (config: any) => void;
}

const ConfigModal = (props: ConfigModalProps) => {
  let [memory, setMemory] = createSignal(props.config.max_memory);
  let [java, setJava] = createSignal(props.config.java_executable || "");
  let [dir, setDir] = createSignal(props.config.game_directory);
  let [fadeOut, setFadeOut] = createSignal(false);

  const handleClose = () => {
    setFadeOut(true);
    setTimeout(() => props.onClose(), 300);
  }
  const handleSave = () => {
    setFadeOut(true);
    setTimeout(() => {
      props.onSave({
        ...props.config,
        max_memory: memory(),
        java_executable: java(),
        game_directory: dir(),
      });
    }, 300);
  };
  return (
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm transition">
    <div
        class={
          "bg-white/90 rounded-3xl shadow-2xl p-8 w-full max-w-md border border-gray-200 flex flex-col items-center " +
          (fadeOut() ? "fade-out" : "animate-fade-in")
        }
      >
        <h2 class="text-3xl font-black text-gray-800 mb-6 tracking-tight">Advanced configuration</h2>
        <div class="w-full flex flex-col gap-4">
          <div>
            <label class="block font-semibold mb-1 text-gray-700">Max memory (MB)</label>
            <input
              class="w-full px-4 py-2 rounded-lg border border-gray-300 focus:ring-2 focus:ring-green-400 outline-none text-lg transition"
              type="number"
              min={512}
              value={memory()}
              onInput={e => setMemory(Number(e.currentTarget.value))}
            />
          </div>
          <div>
            <label class="block font-semibold mb-1 text-gray-700">Java executable</label>
            <input
              class="w-full px-4 py-2 rounded-lg border border-gray-300 focus:ring-2 focus:ring-green-400 outline-none text-lg transition"
              value={java()}
              onInput={e => setJava(e.currentTarget.value)}
              placeholder="java"
            />
          </div>
          <div>
            <label class="block font-semibold mb-1 text-gray-700">Game directory</label>
            <div class="flex items-center gap-2">
            <input
              class="w-full px-4 py-2 rounded-lg focus:ring-2 focus:ring-green-400 outline-none text-lg transition border border-gray-300"
              value={dir()}
              onInput={e => setDir(e.currentTarget.value)}
              />
            <button class="px-4 py-2 rounded-lg hover:bg-gray-300 text-gray-800 font-bold transition ">
              <Folder />
            </button>
              </div>
            </div>
        </div>
        <div class="flex justify-end gap-3 mt-8 w-full">
          <button
            class="px-6 py-2 rounded-lg bg-gray-300 hover:bg-gray-400 text-gray-800 font-bold transition"
            onClick={handleClose}
          >
            Cancel
          </button>
          <button
            class="px-6 py-2 rounded-lg bg-green-500 hover:bg-green-600 text-white font-bold shadow transition"
            onClick={handleSave}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
};

export default ConfigModal;