use std::fs;
use std::path::Path;
use base64::Engine;
use crate::access_manager::{AccessFilter, AccessManager};
use saturn_backend::regions::HexRegion;

fn write_region_contents(destination: &Path, contents: &str) {
    let Ok(value) = base64::engine::general_purpose::STANDARD.decode(contents) else { return };

    fs::write(destination, value).ok();
}

#[tauri::command]
pub async fn export_binary_contents(data: Vec<u8>, filters: Vec<AccessFilter>, state: tauri::State<'_, AccessManager>) -> Result<String, ()> {
    let destination = state.select_save("Save File", &filters, false).await.ok_or(())?;

    fs::write(&destination, data).map_err(|_| ())?;

    Ok(destination.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn export_hex_contents(data: &str, state: tauri::State<'_, AccessManager>) -> Result<String, ()> {
    let destination = state.select_save("Save Region File", &[], false).await.ok_or(())?;

    write_region_contents(&destination, data);

    Ok(destination.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn export_hex_regions(regions: Vec<HexRegion>, state: tauri::State<'_, AccessManager>) -> Result<String, ()> {
    let destination = state.select_save("Save Directory", &[], false).await.ok_or(())?;

    fs::create_dir_all(&destination).ok();

    for region in regions {
        let path = destination.join(format!("{}.txt", region.name));

        write_region_contents(&path, &region.data);
    }

    Ok(destination.to_string_lossy().to_string())
}
