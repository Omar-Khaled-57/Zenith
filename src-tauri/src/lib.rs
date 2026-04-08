mod sensor;

use sensor::SensorEngine;
use std::time::Duration;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut engine = SensorEngine::new();
                loop {
                    let payload = engine.poll();
                    // tauri::Emitter trait must be in scope for .emit()
                    use tauri::Emitter;
                    let _ = app_handle.emit("system-metrics", &payload);
                    std::thread::sleep(Duration::from_millis(1000));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
