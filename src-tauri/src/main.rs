// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod minecraft_api;
fn main() {
    let base_dir = "./.minecraft_fake";
    let json = minecraft_api::download_version_manifest().unwrap();
    let manifest = minecraft_api::parse_version_manifest(&json).unwrap();
    let first_version = &manifest.versions[0];
    let version_json = minecraft_api::download_version_json(&first_version.url).unwrap();
    let version_json = minecraft_api::parse_version_json(&version_json).unwrap();
    println!("main_class: {}", version_json.mainClass);
    println!("Assets: {}", version_json.assetIndex.id);
    println!("First 3 libraries");
    for library in version_json.libraries.iter().take(3) {
        println!("{}", library.name);
    }

    let jar_path = format!("{}/versions/{}/{}.jar", base_dir, first_version.id, first_version.id);
    let jar_url = &version_json.downloads.client.url;
    minecraft_api::download_file(jar_url, &jar_path).unwrap();
    println!("Jar downloaded to: {}", jar_path);
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![launch_minecraft])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");

}

#[tauri::command]
fn launch_minecraft() -> Result<(), String> {
    use std::process::Command;
    Command::new("explorer.exe")
        .arg(r"C:\Users\nacho\Desktop\Minecraft Launcher.lnk")
        .spawn()
        .map_err(|e| format!("Error al lanzar el acceso directo de Minecraft: {}", e))?;
    Ok(())
}