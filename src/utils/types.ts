// Minecraft version types
export interface Version {
    id: string;
    type: string;
    url: string;
}

// Launcher configuration types
export interface LauncherConfig {
    player_name: string;
    player_uuid?: string;
    game_directory: string;
    java_executable?: string;
    jvm_args?: string[];
    memory_max?: number;
    memory_min?: number;
}

// Application state types
export interface LauncherState {
    logs: string[];
    progress: number;
    showAll: boolean;
    selectedVersion: string;
    showVersionSelector: boolean;
    versions: Version[];
    config: LauncherConfig;
    playerName: string;
    activeSection: string;
}

// Event payload types
export interface LogEvent {
    payload: string;
}

export interface ProgressEvent {
    payload: number;
}

// Component prop types
export interface SidebarProps {
    activeSection: string;
    onSectionChange: (section: string) => void;
}

export interface ConfigPageProps {
    config: LauncherConfig;
    playerName: string;
    setPlayerName: (name: string) => void;
    onSave: (config: LauncherConfig) => void;
}

export interface ListVersionsProps {
    versions: Version[];
    selectedVersion: string;
    setSelectedVersion: (version: string) => void;
}

// Utility types
export type SectionType = "Home" | "Settings";
export type VersionType = "release" | "snapshot" | "old_beta" | "old_alpha";

// Error types
export interface LauncherError {
    message: string;
    code?: string;
    details?: any;
}