import { createSignal } from "solid-js";

type ListVersionsProps = {
  selectedVersion: string;
  setSelectedVersion: (v: string) => void;
};

const allReleases = ["1.20.1", "1.20", "1.19.2"];
const allSnapshots = ["1.20-rc1", "1.20-pre7", "1.19.3-rc2"];

const ListVersions = (props: ListVersionsProps) => {
  const [showSnapshots, setShowSnapshots] = createSignal(false);

  const versions = () =>
    showSnapshots() ? [...allReleases, ...allSnapshots] : allReleases;

  return (
    <div class="bg-white/90 dark:bg-gray-900/90 rounded-xl shadow-lg p-4 border border-gray-300 dark:border-gray-800 w-full animate-fade-in backdrop-blur-md">
      <div class="flex items-center justify-between mb-3">
        <span class="font-bold text-gray-800 dark:text-gray-100 text-lg">
          Selecciona versión
        </span>
        <button
          class="text-xs px-2 py-1 rounded bg-blue-500 hover:bg-blue-600 text-white"
          onClick={() => setShowSnapshots((v) => !v)}
        >
          {showSnapshots() ? "Hide Snapshots" : "Show Snapshots"}
        </button>
      </div>
      <ul class="max-h-56 overflow-y-auto space-y-1">
        {versions().map((version) => (
          <li>
            <button
              class={`flex items-center w-full text-left px-4 py-2 rounded-lg transition ${
                props.selectedVersion === version
                  ? "bg-green-100 dark:bg-green-700 font-bold"
                  : "hover:bg-green-50 dark:hover:bg-green-800"
              }`}
              onClick={() => props.setSelectedVersion(version)}
            >
              {version}
              {props.selectedVersion === version && (
                <span class="ml-auto text-green-600 dark:text-green-300">
                  ✓
                </span>
              )}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default ListVersions;
