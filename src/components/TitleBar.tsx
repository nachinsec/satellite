import { getCurrentWindow } from "@tauri-apps/api/window";
export default function TitleBar() {
  return (
    <div
      class="flex items-center justify-between px-3 h-10 bg-gray-900 border-b border-gray-200 dark:border-gray-700 select-none"
      style={{
        "-webkit-app-region": "drag",
        "user-select": "none",
        "-webkit-user-select": "none",
      }}
    >
      <div class="flex items-center gap-2">
        <img src="logo.png" alt="Logo" class="w-6 h-6" draggable={false} />
        <span class="text-white font-bold tracking-wide text-lg">
          Satellite
        </span>
      </div>
      <div class="flex gap-1" style={{ "-webkit-app-region": "no-drag" }}>
        <button
          class="w-8 h-8 flex items-center justify-center rounded hover:bg-white/20 text-white transition"
          onClick={() => getCurrentWindow().minimize()}
          aria-label="Minimize"
        >
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            viewBox="0 0 24 24"
          >
            <line
              x1="6"
              y1="18"
              x2="18"
              y2="18"
              stroke="currentColor"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <button
          class="w-8 h-8 flex items-center justify-center rounded hover:bg-red-500/80 text-white transition"
          onClick={() => getCurrentWindow().close()}
          aria-label="Close"
        >
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            viewBox="0 0 24 24"
          >
            <line
              x1="6"
              y1="6"
              x2="18"
              y2="18"
              stroke="currentColor"
              stroke-linecap="round"
            />
            <line
              x1="6"
              y1="18"
              x2="18"
              y2="6"
              stroke="currentColor"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
