// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod minecraft_api;
use glob::glob;
use std::process::Command;
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

    for lib in &version_json.libraries {
        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                if let Some(lib_path_rel) = &artifact.path {
                    let lib_path = format!("{}/libraries/{}", base_dir, lib_path_rel);
                    let lib_url = &artifact.url;
                    if !std::path::Path::new(&lib_path).exists() {
                        minecraft_api::download_file(lib_url, &lib_path).unwrap();
                        println!("Descargada librería: {}", lib_path);
                    } else {
                        println!("Ya existe: {}", lib_path);
                    }
                }
            }
        }
    }

    let assets_index_path = format!(
        "{}/assets/indexes/{}.json",
        base_dir,
        version_json.assetIndex.id,
    );
    minecraft_api::download_assets(&assets_index_path, base_dir).unwrap();
    println!("Assets index downloaded to: {}", assets_index_path);


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
    .arg(&version_json.mainClass)
    .arg("--username").arg("Player")
    .arg("--version").arg(&first_version.id)
    .arg("--gameDir").arg(base_dir)
    .arg("--assetsDir").arg(format!("{}/assets", base_dir))
    .arg("--assetIndex").arg(&version_json.assetIndex.id)
    .arg("--uuid").arg("N/A")
    .arg("--accessToken").arg("N/A")
    .arg("--userType").arg("legacy")
    .arg("--versionType").arg("release")
    .spawn()
    .expect("Error al lanzar Minecraft");
    println!("Comando Java lanzado, revisa si se abre la ventana de Minecraft.");


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