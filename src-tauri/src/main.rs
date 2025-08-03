// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod downloader;
mod errors;
mod launcher;
mod minecraft_api;
mod mods;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            start_launcher,
            get_versions,
            get_config,
            update_config,
            validate_config,
            get_system_info,
            get_installed_mods,
            toggle_mod,
            delete_mod,
            install_mod_from_file,
            search_mods_online,
            install_mod_online
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
