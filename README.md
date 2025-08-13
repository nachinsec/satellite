# Satellite

A modern and minimal Minecraft launcher built with Tauri + SolidJS.

---

## What is Satellite?

Satellite is a lightweight launcher to manage and start your Minecraft versions. It lets you configure player name, game directory, Java and memory, and ships basic integrated mod management (install from file, search on Modrinth, enable/disable, and delete mods).

## Features

- Choose and launch Minecraft versions
- Edit player name and game directory
- Configure Java executable and memory settings
- Mods: list installed, enable/disable, delete
- Modrinth search and install from .jar file
- Clean interface with dark theme support

---

## System requirements

- OS: Windows 10/11, Linux, or macOS (Intel/ARM)
- Java: JRE/JDK 17 or newer installed and available on your system
- RAM: 4 GB minimum (8 GB recommended for mods)
- GPU: OpenGL 4.4 or newer compatible
- Storage: 2–4 GB for vanilla; 6 GB+ for modpacks

Note: On Windows, you can select the Java executable from the launcher settings.

---

## Requirements to play WITHOUT mods (Vanilla)

1. Have Java 17+ installed.
2. Internet connection for the first version download.
3. Enough disk space for the selected version.

In Satellite:
- Open `Settings` and adjust player name, game directory, and memory.
- Pick the Minecraft version under `Versions` and click `Play`.

---

## Requirements to play WITH mods

1. Java 17+ installed and more memory available (6–8 GB recommended for modpacks).
2. Install a mod loader compatible with your mods:
   - Fabric: https://fabricmc.net
   - Forge: https://files.minecraftforge.net
   - Quilt: https://quiltmc.org (required by some mods)
3. Ensure mods match your Minecraft version and the chosen loader (Fabric/Forge/Quilt).

In Satellite (mod management):
- `Mods` → `Installed Mods`: see local mods and enable/disable or delete.
- `Mods` → `Browse Mods`: search Modrinth and download a compatible `.jar`.
- Or use `Install from file` to add a previously downloaded `.jar`.

Important: Satellite does not install Fabric/Forge automatically. Use the official installers and select the version/profile with the corresponding loader.

---

## Installation (end users)

There are two ways:

- Binary/installer: generate an installer with `npm run tauri build` (see development section). If GitHub releases are available, download from the `Releases` tab.
- Development: run in dev mode (see below) and use the launcher directly.

---

## Quick start

- Start vanilla:
  1) Open Satellite → Configure name, directory, and memory.
  2) Select a Minecraft version.
  3) Click `Play`.

- Start with mods:
  1) Install Fabric/Forge/Quilt for your MC version.
  2) Open Satellite → `Mods` → install/search compatible mods.
  3) Select the version/profile with the loader and click `Play`.

---

## Development (contributors)

1. Install frontend dependencies:
   ```bash
   npm install
   ```

2. Run in development mode (Tauri + frontend):
   ```bash
   npm run tauri dev
   ```

3. Build a release installer:
   ```bash
   npm run tauri build
   ```

---

## Technologies

- **SolidJS** (UI framework)
- **Tauri** (native desktop backend, Rust)
- **Rust** (backend logic)
- **Vite** (build tool)
- **TailwindCSS** (styling)
- **solid-toast** (notifications)

Project is under active development.

---

## FAQ

- Where is the `mods` folder?
  - By default, inside the configured game directory (e.g., `.minecraft/mods`). Satellite detects it automatically when you install mods.

- Why doesn’t the game start with a mod?
  - Ensure the mod version matches your Minecraft version and loader (Fabric/Forge/Quilt). Increase assigned memory if needed.

- Can I change the Java runtime?
  - Yes, you can select the Java executable from Settings.

---

## License

This project is distributed under the license included in `LICENSE`.
