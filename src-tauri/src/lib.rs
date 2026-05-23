mod sensor;

use sensor::SensorEngine;
use std::time::Duration;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            std::thread::Builder::new()
                .name("sensor-poll".into())
                .spawn(move || {
                    let mut engine = SensorEngine::new();
                    loop {
                        let payload = engine.poll();
                        if let Err(e) = app_handle.emit("system-metrics", &payload) {
                            eprintln!("Failed to emit metrics: {e}");
                        }
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                })
                .expect("Failed to spawn sensor thread");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
