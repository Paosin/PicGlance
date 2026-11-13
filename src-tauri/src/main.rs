#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

pub mod contracts;
pub mod editor;
pub mod viewer;
mod viewer_host;
mod compression_host;
#[cfg(windows)]
mod desktop;

#[cfg(windows)]
#[tauri::command]
fn desktop_close_from_host(window: tauri::WebviewWindow) -> Result<(), String> {
    desktop::close_from_host(&window).map_err(|error| error.to_string())
}

#[tauri::command]
async fn editor_open(app: tauri::AppHandle, document_id: String) -> Result<String, contracts::ContractError> {
    use contracts::{ContractError, ErrorCode};

    tauri::async_runtime::spawn_blocking(move || {
        let input = app.state::<viewer::Viewer>().editor_input(&document_id)
            .map_err(|_| ContractError { code: ErrorCode::InvalidInput })?;
        app.state::<editor::ipc::EditorState>().open(input)
    })
    .await
    .map_err(|_| ContractError { code: ErrorCode::Io })?
}

#[tauri::command]
fn windows_build() -> u32 {
    #[cfg(windows)]
    {
        use windows_sys::{Wdk::System::SystemServices::RtlGetVersion, Win32::System::SystemInformation::OSVERSIONINFOW};
        let mut version: OSVERSIONINFOW = unsafe { std::mem::zeroed() };
        version.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
        if unsafe { RtlGetVersion(&mut version) } == 0 {
            return version.dwBuildNumber;
        }
    }
    0
}

#[cfg(feature = "first-image-diagnostic")]
mod diagnostic {
    use std::{fs::File, io::Read, path::PathBuf};

    pub struct Input {
        image: PathBuf,
        marker: PathBuf,
    }

    impl Input {
        pub fn from_args() -> Option<Self> {
            let mut args = std::env::args_os().skip(1);
            if args.next()? != "--first-image-diagnostic" {
                return None;
            }
            Some(Self { image: args.next()?.into(), marker: args.next()?.into() })
        }
    }

    #[tauri::command]
    pub async fn diagnostic_image(input: tauri::State<'_, Option<Input>>) -> Result<tauri::ipc::Response, String> {
        let Some(input) = input.inner() else { return Ok(tauri::ipc::Response::new(Vec::new())) };
        let file = File::open(&input.image).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.take(80_000_001).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() > 80_000_000 || !bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            return Err("Expected a JPEG up to 80 MB".into());
        }
        Ok(tauri::ipc::Response::new(bytes))
    }

    #[tauri::command]
    pub fn diagnostic_presented(input: tauri::State<'_, Option<Input>>) -> Result<(), String> {
        if let Some(input) = input.inner() {
            File::create_new(&input.marker).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[tauri::command]
fn set_webview_memory_target(window: tauri::WebviewWindow, low: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        };
        use windows_core::Interface;

        window.with_webview(move |webview| unsafe {
            if let Ok(core) = webview.controller().CoreWebView2() {
                // Older WebView2 runtimes can continue with their default policy.
                if let Ok(core) = core.cast::<ICoreWebView2_19>() {
                    let target = if low { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW }
                        else { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL };
                    let _ = core.SetMemoryUsageTargetLevel(target);
                }
            }
        }).map_err(|error| error.to_string())?;
    }
    #[cfg(not(windows))]
    let _ = (window, low);
    Ok(())
}

fn main() {
    #[cfg(windows)]
    if let Some(code) = desktop::handle_installer_command() {
        std::process::exit(code);
    }
    let builder = tauri::Builder::default();
    #[cfg(windows)]
    let builder = builder.plugin(desktop::single_instance());
    let builder = builder
        .plugin(tauri_plugin_dialog::init())
        .manage(viewer::Viewer::default())
        .manage(viewer_host::PendingViewerOpens::default())
        .manage(contracts::EditedResources::default())
        .manage(editor::ipc::EditorState::default())
        .manage(compression_host::CompressionState::default())
        .on_webview_event(|webview, event| {
            if webview.label() == "main" {
                if let tauri::WebviewEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                    viewer_host::queue_paths(webview.app_handle(), paths.iter().cloned());
                }
            }
        });
    #[cfg(windows)]
    let builder = builder.setup(|app| {
        use windows_sys::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE,
        };

        let handle = app.handle().clone();
        desktop::setup(app.handle(), Some(Box::new(move |paths| {
            viewer_host::queue_paths(&handle, paths);
        })))?;
        if let Some(window) = app.get_webview_window("main") {
            let hwnd = window.hwnd()?;
            // Windows 11 can keep the native shadow without painting the frame border.
            let result = unsafe {
                DwmSetWindowAttribute(
                    hwnd.0,
                    DWMWA_BORDER_COLOR as u32,
                    std::ptr::from_ref(&DWMWA_COLOR_NONE).cast(),
                    std::mem::size_of::<u32>() as u32,
                )
            };
            if result < 0 {
                // Older Windows versions lack the border-color attribute.
                window.set_shadow(false)?;
            }
        }
        Ok(())
    });
    #[cfg(not(windows))]
    let builder = builder.setup(|app| {
        viewer_host::queue_startup(app.handle());
        Ok(())
    });
    #[cfg(feature = "first-image-diagnostic")]
    let builder = builder.manage(diagnostic::Input::from_args());
    let builder = builder.invoke_handler(tauri::generate_handler![
        windows_build, set_webview_memory_target,
        viewer::viewer_open, viewer::viewer_directory, viewer::viewer_thumbnail, viewer::viewer_resource,
        viewer::viewer_cancel, viewer::viewer_copy, viewer::viewer_paste, viewer::viewer_choose,
        viewer_host::viewer_take_pending_open,
        viewer::viewer_rename,
        editor_open, editor::ipc::editor_render, editor::ipc::editor_save,
        editor::ipc::editor_export, editor::ipc::editor_close,
        compression_host::compression_reserve, compression_host::compression_run,
        compression_host::compression_cancel, compression_host::compression_read,
        compression_host::compression_save,
        #[cfg(windows)]
        desktop::desktop_read,
        #[cfg(windows)]
        desktop::desktop_write,
        #[cfg(windows)]
        desktop::desktop_set_tray,
        #[cfg(windows)]
        desktop::desktop_associations,
        #[cfg(windows)]
        desktop::desktop_set_associations,
        #[cfg(windows)]
        desktop::desktop_default_apps,
        #[cfg(windows)]
        desktop::desktop_check_update,
        #[cfg(windows)]
        desktop::desktop_open_releases,
        #[cfg(windows)]
        desktop::desktop_open_license,
        #[cfg(windows)]
        desktop_close_from_host,
        #[cfg(feature = "first-image-diagnostic")]
        diagnostic::diagnostic_image,
        #[cfg(feature = "first-image-diagnostic")]
        diagnostic::diagnostic_presented,
    ]);
    let mut context = tauri::generate_context!();
    #[cfg(windows)]
    desktop::configure_context(&mut context).expect("configure PicGlance desktop");
    builder.run(context).expect("start PicGlance");
}
