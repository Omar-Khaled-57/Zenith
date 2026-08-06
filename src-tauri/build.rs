fn main() {
    let mut attrs = tauri_build::Attributes::new();
    #[cfg(target_os = "windows")]
    {
        let manifest = tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));
        attrs = attrs.windows_attributes(manifest);
    }
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
