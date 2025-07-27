import { useTheme } from "../theme";

const navItems = [
  { label: "Home", icon: "🏠" },
  { label: "Instances", icon: "📦" },
  { label: "Mods", icon: "🧩" },
  { label: "Settings", icon: "⚙️" },
];

type SidebarProps = {
  activeSection: string;
  onSectionChange: (section: string) => void;
};

export default function Sidebar(props: SidebarProps) {
  return (
    <aside class="sidebar h-screen w-56 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 flex flex-col shadow-lg">
      <div class="flex flex-col gap-2 mt-8">
        {navItems.map((item) => (
          <button
            class={`flex items-center gap-3 px-5 py-3 rounded-lg text-lg font-semibold transition-colors mx-3
              ${props.activeSection === item.label
                ? "bg-green-100 dark:bg-green-800 text-green-700 dark:text-green-200"
                : "text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800"}
            `}
            onClick={() => props.onSectionChange(item.label)}
          >
            <span class="text-xl">{item.icon}</span>
            {item.label}
          </button>
        ))}
      </div>
      <div class="flex-1" />
      <div class="p-4 flex flex-col gap-3">
        <ThemeSwitch />
        <div class="text-xs text-gray-400 dark:text-gray-600 mt-2">Satellite Launcher © 2025</div>
      </div>
    </aside>
  );
}

function ThemeSwitch() {
  const { theme, toggleTheme } = useTheme();
  return (
    <button
      class="w-full flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm font-semibold bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200"
      onClick={toggleTheme}
      aria-label="Cambiar tema"
    >
      {theme() === "dark" ? "🌙 Dark" : "☀️ Light"}
    </button>
  );
}
