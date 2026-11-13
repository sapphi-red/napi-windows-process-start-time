fn main() {
    let target = std::env::var("TARGET").expect("Cargo must set TARGET");
    assert!(
        matches!(
            target.as_str(),
            "x86_64-pc-windows-msvc" | "aarch64-pc-windows-msvc"
        ),
        "Only Windows x64 and ARM64 MSVC targets are supported"
    );
    // There is no CRT initialization: this addon uses only core and Win32.
    println!("cargo:rustc-link-arg=/ENTRY:DllMain");
    // DllMain is intentionally both the PE entry point and an exported symbol.
    println!("cargo:rustc-link-arg=/IGNORE:4216");
    println!("cargo:rerun-if-changed=build.rs");
}
