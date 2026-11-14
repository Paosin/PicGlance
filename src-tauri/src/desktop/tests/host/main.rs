#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[path = "../../../contracts/mod.rs"]
#[allow(dead_code)]
mod contracts;
#[path = "../../mod.rs"]
mod desktop;

#[tauri::command]
fn fixture_image(path: String) -> Result<tauri::ipc::Response, String> {
    let path = std::path::Path::new(&path);
    if !path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with("PicGlance-test-"))
        || !path.starts_with(std::env::temp_dir())
    {
        return Err("Expected a desktop test fixture in TEMP".into());
    }
    std::fs::read(path).map(tauri::ipc::Response::new).map_err(|error| error.to_string())
}

fn main() {
    if let Some(code) = desktop::handle_installer_command() { std::process::exit(code); }
    let mut context = tauri::generate_context!();
    desktop::configure_context(&mut context).expect("desktop context");
    tauri::Builder::default()
        .plugin(desktop::single_instance())
        .setup(|app| desktop::setup(app.handle(), None))
        .invoke_handler(tauri::generate_handler![
            desktop::desktop_read, desktop::desktop_write, desktop::desktop_take_paths,
            desktop::desktop_set_tray, desktop::desktop_associations, desktop::desktop_set_associations,
            desktop::desktop_default_apps, desktop::desktop_check_update, desktop::desktop_open_releases,
            desktop::desktop_open_license, fixture_image
        ])
        .run(context)
        .expect("start desktop test host");
}
