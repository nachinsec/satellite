// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod minecraft_api;
use glob::glob;
use std::process::Command;
use tauri::Emitter;
use crate::minecraft_api::MinecraftVersion;
use reqwest::Client;
fn main() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![start_launcher, get_versions])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
async fn get_versions() -> Result<Vec<MinecraftVersion>, String> {
    let client = Client::new();
    let manifest_json = minecraft_api::download_version_manifest(&client).await.map_err(|e| e.to_string())?;
    let manifest = minecraft_api::parse_version_manifest(&manifest_json).map_err(|e| e.to_string())?;
    let versions = manifest.versions.iter()
    .map(|v| MinecraftVersion {
        id: v.id.clone(),
        r#type: v.r#type.clone(),
        url: v.url.clone(),
    })
    .collect();
    Ok(versions)
}


#[tauri::command]
async fn start_launcher(window: tauri::Window){
    let base_dir = "./.minecraft_fake";
    let client = Client::new();

    window.emit("log", "Descargando manifiesto de versiones...").ok();
    let manifest_json = minecraft_api::download_version_manifest(&client).await.unwrap();
    let manifest = minecraft_api::parse_version_manifest(&manifest_json).unwrap();

    let first_version = &manifest.versions[0];
    window.emit("log", format!("Descargando JSON de versión: {}", first_version.id)).ok();
    let version_json_str = minecraft_api::download_version_json(&client, &first_version.url).await.unwrap();
    let version_json = minecraft_api::parse_version_json(&version_json_str).unwrap();

    window.emit("log", "Descargando JAR principal...").ok();
    let jar_path = format!("{}/versions/{}/{}.jar", base_dir, first_version.id, first_version.id);
    let jar_url = &version_json.downloads.client.url;
    minecraft_api::download_file(&client, jar_url, &jar_path).await.unwrap();
    window.emit("log", format!("Jar descargado en: {}", jar_path)).ok();

    // Descarga librerías (secuencial, puedes paralelizar igual que assets)
    for lib in &version_json.libraries {
        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                if let Some(lib_path_rel) = &artifact.path {
                    let lib_path = format!("{}/libraries/{}", base_dir, lib_path_rel);
                    if !std::path::Path::new(&lib_path).exists() {
                        minecraft_api::download_file(&client, &artifact.url, &lib_path).await.unwrap();
                        window.emit("log", format!("Downloaded library: {}", lib_path)).ok();
                    } else {
                        window.emit("log", format!("Already exists: {}", lib_path)).ok();
                    }
                }
            }
        }
    }

    window.emit("log", "Downloading assets in parallel...").ok();
    let assets_index_path = format!("{}/assets/indexes/{}.json", base_dir, version_json.asset_index.id);
    minecraft_api::download_file(&client, &version_json.asset_index.url, &assets_index_path).await.unwrap();

    let asset_futures = minecraft_api::download_assets(&client, &assets_index_path, base_dir, window.clone());
    asset_futures.await;

    window.emit("log", "All assets downloaded! Launching Minecraft...").ok();


    let mut classpath = Vec::new();
    let pattern = format!("{}/libraries/**/*.jar", base_dir);
    for entry in glob(&pattern).unwrap().filter_map(Result::ok) {
        classpath.push(entry.to_str().unwrap().to_string());
    }

    classpath.push(jar_path.clone());
    let classpath_str = classpath.join(";");
    let output = Command::new("java")
    .arg("-cp")
    .arg(&classpath_str)
    .arg(&version_json.main_class)
    .arg("--username").arg("Player")
    .arg("--version").arg(&first_version.id)
    .arg("--gameDir").arg(base_dir)
    .arg("--assetsDir").arg(format!("{}/assets", base_dir))
    .arg("--assetIndex").arg(&version_json.asset_index.id)
    .arg("--uuid").arg("N/A")
    .arg("--accessToken").arg("N/A")
    .arg("--userType").arg("legacy")
    .arg("--versionType").arg("release")
    .spawn()
    .expect("Error al lanzar Minecraft");
    window.emit("log", format!("Comando Java lanzado, revisa si se abre la ventana de Minecraft.")).unwrap();
}

