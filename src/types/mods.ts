export interface ModInfo {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  minecraft_version: string;
  mod_loader: ModLoader;
  file_name: string;
  file_size: number;
  enabled: boolean;
  dependencies: string[];
  source: ModSource;
}

export enum ModLoader {
  Forge = "Forge",
  Fabric = "Fabric",
  Quilt = "Quilt",
  NeoForge = "NeoForge",
}

export type ModSource = 
  | { Local: null }
  | { Modrinth: { project_id: string } }
  | { CurseForge: { project_id: number } };

export interface ModSearchResult {
  id: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  icon_url?: string;
  minecraft_versions: string[];
  mod_loaders: ModLoader[];
  source: ModSource;
}

export interface ModsPageProps {
  gameDirectory: string;
  minecraftVersion: string;
}
