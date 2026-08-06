mod sensor;
mod worker_supervisor;

use sensor::SensorEngine;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager};
use worker_supervisor::{resolve_worker_exe, WorkerSupervisor};

#[tauri::command]
fn restart_elevated(app: tauri::AppHandle) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_w: Vec<u16> = exe
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let verb_w: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();

    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            verb_w.as_ptr(),
            exe_w.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        return Err(format!("failed to start elevated (code {})", result as isize));
    }
    // In release, the fresh elevated process replaces this one, so exit.
    // In debug (`tauri dev`) exiting would tear down the Vite dev server the
    // relaunched window needs, so keep this process alive and hide its window.
    #[cfg(debug_assertions)]
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }
    #[cfg(not(debug_assertions))]
    {
        app.exit(0);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![restart_elevated])
        .setup(move |app| {
            if let Some(icon) = app.default_window_icon() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon.clone());
                }
            }
            let app_handle = app.handle().clone();
            std::thread::Builder::new()
                .name("sensor-poll".into())
                .spawn(move || {
                    let mut supervisor = WorkerSupervisor::new();
                    supervisor.start(&resolve_worker_exe(&app_handle));
                    let mut engine = SensorEngine::new();
                    while running_clone.load(Ordering::Relaxed) {
                        let latest = supervisor.latest();
                        let worker_error = supervisor.last_error();
                        let payload = engine.poll(&latest, worker_error.as_deref());
                        if app_handle.emit("system-metrics", &payload).is_err() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                    supervisor.stop();
                })
                .expect("Failed to spawn sensor thread");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_handle, event| {
            if matches!(event, tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit) {
                running.store(false, Ordering::Relaxed);
            }
        });
}
