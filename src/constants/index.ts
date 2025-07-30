// Application constants
export const APP_CONSTANTS = {
  MAX_LOGS: 50,
  DEFAULT_VERSION: "1.20.1",
  DEFAULT_PLAYER_NAME: "Player",
  LOG_CONTAINER_HEIGHT: "h-40",
  PROGRESS_TRANSITION: "width 0.2s",
} as const;

// UI Constants
export const UI_CONSTANTS = {
  SECTIONS: {
    HOME: "Home",
    SETTINGS: "Settings",
  },
  LOGO: {
    SIZE: "w-24 h-24",
    ALT: "Satellite Logo",
    SRC: "logo.png",
  },
} as const;

// CSS Classes
export const CSS_CLASSES = {
  BUTTON: {
    PRIMARY: "flex-1 flex items-center gap-2 font-bold py-3 px-6 rounded-lg shadow-lg transition text-lg text-white bg-green-500 hover:bg-green-600 dark:bg-green-500 dark:hover:bg-green-600",
    SECONDARY: "bg-gray-500 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded transition mt-4",
    VERSION_SELECTOR: "bg-green-500 hover:bg-green-600 text-white rounded-lg shadow-lg transition flex items-center px-3",
  },
  CONTAINER: {
    MAIN: "flex-1 flex flex-col items-center justify-center bg-gradient-to-br from-green-100 via-blue-50 to-white dark:from-gray-900 dark:via-gray-800 dark:to-gray-900 p-4 relative",
    LOG: "bg-black/80 rounded-lg p-4 font-mono whitespace-pre-wrap break-all text-sm text-green-100 shadow-inner h-40 overflow-y-auto overflow-x-hidden border border-gray-700",
    PROGRESS_BAR: "w-full mt-4 bg-gray-200 rounded-full h-2.5 dark:bg-gray-700",
    PROGRESS_FILL: "bg-green-500 h-2.5 rounded-full dark:bg-green-600",
  },
  TEXT: {
    TITLE: "text-4xl font-extrabold text-gray-800 dark:text-gray-100 tracking-tight mb-2",
  },
} as const;
