import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { listen } from "@tauri-apps/api/event";
import ListVersions from "./components/ListVersions";
import TitleBar from "./components/TitleBar";

type Version = {
  id: string;
  type: string;
  url: string;
};
function App() {
  const [logs, setLogs] = createSignal<string[]>([]);
  const [progress, setProgress] = createSignal(0);
  const [showAll, setShowAll] = createSignal(false);
  const [selectedVersion, setSelectedVersion] = createSignal("1.20.1");
  const [showVersionSelector, setShowVersionSelector] = createSignal(false);
  const [versions, setVersions] = createSignal<Version[]>([]);
  const MAX_LOGS = 50;
  let logRef: HTMLDivElement | undefined;

  async function startLauncher() {
    await invoke("start_launcher");
  }

  async function getVersions() {
    const versions = await invoke("get_versions");
    setVersions(versions as Version[]);
  }

  onMount(() => {
    getVersions();
  });

  listen("log", (event) => {
    setLogs((logs) => [...logs, event.payload as string]);
    logRef?.scrollTo({ top: logRef.scrollHeight });
  });

  listen("progress", (event) => {
    setProgress(event.payload as number);
  });

  function getLog(): string {
    const all = logs();
    if (showAll()) return all.join("\n");
    return all.slice(-MAX_LOGS).join("\n");
  }

  return (
    <>
      <TitleBar />
      <main class="h-screen flex flex-col items-center justify-center bg-gradient-to-br from-green-100 via-blue-50 to-white p-4">
        <img
          src="logo.png"
          alt="Satellite Logo"
          class="w-24 h-24 mb-2 drop-shadow-lg"
        />
        <h1 class="text-4xl font-extrabold text-gray-800 tracking-tight mb-2">
          Satellite
        </h1>
        <div class="relative w-full max-w-sm flex">
          <button
            class="flex-1 flex items-center gap-2 bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded-lg shadow-lg transition text-lg"
            onClick={startLauncher}
            style={{
              "border-top-right-radius": "0",
              "border-bottom-right-radius": "0",
            }}
          >
            Launch {selectedVersion()}
          </button>
          <button
            class="bg-green-500 hover:bg-green-600 text-white rounded-lg shadow-lg transition flex items-center px-3"
            style={{
              "border-top-left-radius": "0",
              "border-bottom-left-radius": "0",
            }}
            onClick={() => setShowVersionSelector((v) => !v)}
            aria-label="Select version"
          >
            <svg
              class="w-5 h-5"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </button>
          {showVersionSelector() && (
            <div class="absolute z-20 mt-[50px] w-full left-0">
              <ListVersions
                versions={versions()}
                selectedVersion={selectedVersion()}
                setSelectedVersion={(v: string) => {
                  setSelectedVersion(v);
                  setShowVersionSelector(false);
                }}
              />
            </div>
          )}
        </div>
        <div class="w-full mt-4">
          <div
            class="bg-black/80 rounded-lg p-4 font-mono text-sm text-green-100 shadow-inner h-40 overflow-y-auto border border-gray-700"
            ref={logRef}
          >
            {getLog()}
          </div>
        </div>
        <div class="w-full mt-4 bg-gray-200 rounded-full h-2.5 dark:bg-gray-700">
          <div
            class="bg-green-500 h-2.5 rounded-full dark:bg-green-600"
            style={{
              width: `${progress() * 100}%`,
              transition: "width 0.2s",
            }}
          />
        </div>
        <button
          class="bg-gray-500 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded transition mt-4"
          onClick={() =>
            setShowAll((v) => {
              logRef?.scrollTo({ top: logRef.scrollHeight });
              return !v;
            })
          }
        >
          {showAll() ? "Hide" : "Show All"}
        </button>
      </main>
    </>
  );
}

export default App;
