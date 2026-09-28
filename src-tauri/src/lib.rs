use std::fs;
use encoding_rs::WINDOWS_1252;


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn process_form_payload(export_apogee: &str, export_scodoc: &str, bareme: &str, separateur_csv: &str) -> Result<String, String> {
    let bytes = std::fs::read(export_apogee)
    .map_err(|e| format!("Failed to read {}: {}", export_apogee, e))?;

    let content = match String::from_utf8(bytes.clone()) {
        Ok(content) => content,
        Err(_) => {
            let (content, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
            content.into_owned()
        }
    };

    println!("File content:\n{}", content);

    Ok(format!(
        "{0}, this is {1}. {1}, this is {0}",
        "Alice", "Bob"
    ))
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![process_form_payload])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
