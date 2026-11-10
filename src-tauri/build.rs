fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        // Oboe aporta código C++ a la única biblioteca JNI de la aplicación.
        println!("cargo:rustc-link-arg-cdylib=-lc++_static");
        println!("cargo:rustc-link-arg-cdylib=-lc++abi");
        println!("cargo:rustc-link-arg-cdylib=-Wl,--no-undefined");
    }
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("windows-app-manifest.xml"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attributes).expect("falló la preparación de Tauri");
}
