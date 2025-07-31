import "./App.css";
import ListVersions from "./components/ListVersions";
import TitleBar from "./components/TitleBar";
import Sidebar from "./components/Sidebar";
import { ConfigPage } from "./components/ConfigPage";
import { ModsPage } from "./components/ModsPage";
import { Toaster } from "solid-toast";
import { useLauncher } from "./hooks/useLauncher";
import { UI_CONSTANTS, CSS_CLASSES } from "./constants";

function App() {
  const launcher = useLauncher();

  return (
    <>
      <Toaster/>
      <TitleBar />
      <div class="flex h-screen">
        <Sidebar
          activeSection={launcher.activeSection()}
          onSectionChange={launcher.setActiveSection}
        />
        <main class={CSS_CLASSES.CONTAINER.MAIN}>
          {launcher.activeSection() === UI_CONSTANTS.SECTIONS.SETTINGS ? (
            <ConfigPage
              config={launcher.config()}
              playerName={launcher.playerName()}
              setPlayerName={launcher.setPlayerName}
              onSave={launcher.updateConfig}
            />
          ) : launcher.activeSection() === "Mods" ? (
            <ModsPage
              gameDirectory={launcher.config()?.game_directory || ""}
              minecraftVersion={launcher.selectedVersion()}
            />
          ) : (
            <>
              <img
                src={UI_CONSTANTS.LOGO.SRC}
                alt={UI_CONSTANTS.LOGO.ALT}
                class={`${UI_CONSTANTS.LOGO.SIZE} mb-2 drop-shadow-lg`}
              />
              <h1 class={CSS_CLASSES.TEXT.TITLE}>
                Satellite
              </h1>
              <div class="relative w-full max-w-sm flex">
                <button
                  class={CSS_CLASSES.BUTTON.PRIMARY}
                  onClick={launcher.startLauncher}
                  style={{
                    "border-top-right-radius": "0",
                    "border-bottom-right-radius": "0",
                  }}
                >
                  Launch {launcher.selectedVersion()}
                </button>
                <button
                  class={CSS_CLASSES.BUTTON.VERSION_SELECTOR}
                  style={{
                    "border-top-left-radius": "0",
                    "border-bottom-left-radius": "0",
                  }}
                  onClick={() => launcher.setShowVersionSelector(v => !v)}
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
                {launcher.showVersionSelector() && (
                  <div class="absolute z-20 mt-[50px] w-full left-0">
                    <ListVersions
                      versions={launcher.versions()}
                      selectedVersion={launcher.selectedVersion()}
                      setSelectedVersion={launcher.handleVersionSelect}
                    />
                  </div>
                )}
              </div>
              <div class="w-full mt-4">
                <div
                  class={CSS_CLASSES.CONTAINER.LOG}
                  ref={launcher.logRef}
                >
                  {launcher.getLog()}
                </div>
              </div>
              <div class={CSS_CLASSES.CONTAINER.PROGRESS_BAR}>
                <div
                  class={CSS_CLASSES.CONTAINER.PROGRESS_FILL}
                  style={{
                    width: `${launcher.progress() * 100}%`,
                    transition: "width 0.2s",
                  }}
                />
              </div>
              <button
                class={CSS_CLASSES.BUTTON.SECONDARY}
                onClick={launcher.toggleShowAll}
              >
                {launcher.showAll() ? "Hide" : "Show All"}
              </button>
            </>
           )}
        </main>
      </div>
    </>
  );
}

export default App;

