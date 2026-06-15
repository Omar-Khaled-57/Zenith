mod sensor;

use sensor::SensorEngine;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            std::thread::Builder::new()
                .name("sensor-poll".into())
                .spawn(move || {
                    let mut engine = SensorEngine::new();
                    while running_clone.load(Ordering::Relaxed) {
                        let payload = engine.poll();
                        if app_handle.emit("system-metrics", &payload).is_err() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(1000));
                    }
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
