import { createSignal } from "solid-js";

type ListVersionsProps = {
  versions: Version[];
  selectedVersion: string;
  setSelectedVersion: (v: string) => void;
};

type Version = {
  id: string;
  type: string;
  url: string;
};

const ListVersions = (props: ListVersionsProps) => {
  const [showSnapshots, setShowSnapshots] = createSignal(false);

  function filterVersions() {
    if (showSnapshots()) {
      return props.versions;
    } else {
      return props.versions.filter((version) => version.type === "release");
    }
  }


  return (
    <div class="bg-white/90 dark:bg-gray-900/90 rounded-xl shadow-lg p-4 border border-gray-300 dark:border-gray-800 w-full animate-fade-in backdrop-blur-md">
      <div class="flex items-center justify-between mb-3">
        <span class="font-bold text-gray-800 dark:text-gray-100 text-lg">
          Select version
        </span>
        <button
          class="text-xs px-2 py-1 rounded bg-blue-500 hover:bg-blue-600 text-white"
          onClick={() => setShowSnapshots((v) => !v)}
        >
          {showSnapshots() ? "Hide Snapshots" : "Show Snapshots"}
        </button>
      </div>
      <ul class="max-h-24 overflow-y-auto space-y-1">
        {filterVersions().map((version) => (
          <li>
            <button
              class={`flex items-center w-full text-left px-4 py-2 rounded-lg transition ${
                props.selectedVersion === version.id
                  ? "bg-green-100 dark:bg-green-700 font-bold"
                  : "hover:bg-green-50 dark:hover:bg-green-800"
              }`}
              onClick={() => props.setSelectedVersion(version.id)}
            >
              {version.id}
              {props.selectedVersion === version.id && (
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
