import { createSignal, onMount } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { listen } from "@tauri-apps/api/event";
function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");
  const [logs, setLogs] = createSignal<string[]>([]);
  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  async function launchMinecraft() {
    await invoke("launch_minecraft");
  }

  async function startLauncher() {
    await invoke("start_launcher");
  }

  onMount(() => {
    startLauncher();
  });

  listen("log", (event) => {
    setLogs((logs) => [...logs, event.payload as string]);
  });

  return (
    <main class="container">
      <h1>Welcome to Tauri + Solid</h1>

      <div class="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://solidjs.com" target="_blank">
          <img src={logo} class="logo solid" alt="Solid logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and Solid logos to learn more.</p>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg()}</p>

      <button onClick={launchMinecraft}>Launch Minecraft</button>

      <div class="logs">
        {logs().map((log) => (
          <p>{log}</p>
        ))}
      </div>
    </main>
  );
}

export default App;
