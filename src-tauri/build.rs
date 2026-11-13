fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "windows_build", "set_webview_memory_target", "diagnostic_image", "diagnostic_presented",
            "viewer_open", "viewer_directory", "viewer_thumbnail", "viewer_resource", "viewer_cancel",
            "viewer_copy", "viewer_paste", "viewer_choose", "viewer_take_pending_open",
            "viewer_rename",
            "editor_open", "editor_render", "editor_save", "editor_export", "editor_close",
            "compression_reserve", "compression_run", "compression_cancel", "compression_read", "compression_save",
            "desktop_read", "desktop_write", "desktop_set_tray", "desktop_associations",
            "desktop_set_associations", "desktop_default_apps", "desktop_check_update",
            "desktop_open_releases", "desktop_open_license",
            "desktop_close_from_host",
        ]),
    ))
    .expect("build Tauri resources");
}
