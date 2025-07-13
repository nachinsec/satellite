import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { listen } from "@tauri-apps/api/event";
import logo from "./assets/logo.svg";
function App() {
  const [logs, setLogs] = createSignal<string[]>([]);
  const [progress, setProgress] = createSignal(0);
  const [showAll, setShowAll] = createSignal(false);
  const MAX_LOGS = 50;
  let logRef: HTMLDivElement | undefined;
  async function startLauncher() {
    await invoke("start_launcher");
  }

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
    <main class="flex flex-col items-center justify-center h-full w-full gap-4 p-4">
      <h1 class="text-3xl font-bold">Satellite</h1>
      <button
        class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded"
        onClick={startLauncher}
      >
        Launch Minecraft
      </button>
      <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700">
        <div
          class="bg-green-500 h-2.5 rounded-full dark:bg-green-600"
          style={{
            width: `${progress() * 100}%`,
            height: "100%",
            background: "#4caf50",
            "border-radius": "8px",
            transition: "width 0.2s",
          }}
        />
      </div>
      <div
        class="logs w-full"
        style={{
          "overflow-y": "scroll",
          height: "40vh",
          "max-height": "40vh",
          "white-space": "pre-wrap",
          "background-color": "#333",
          color: "#fff",
          "text-align": "left",
          "border-radius": "8px",
          padding: "0.5rem 1rem",
        }}
        ref={logRef}
      >
        {getLog()}
      </div>
      <button
        class="bg-gray-500 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded"
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
  );
}

export default App;
