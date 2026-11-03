use std::fs;
use std::process::Command;

#[tauri::command]
pub fn open_folder_dir(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let win_path = path.replace("/", "\\");
        Command::new("explorer.exe")
            .arg(&win_path)
            .spawn()
            .map_err(|e| format!("Gagal membuka Explorer: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn export_excel_file(path: String, bytes: Vec<u8>) -> Result<(), String> {
    fs::write(&path, bytes).map_err(|e| format!("Gagal menyimpan file ke disk: {}", e))
}