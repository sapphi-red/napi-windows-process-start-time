fn main() {
    let target = std::env::var("TARGET").expect("Cargo must set TARGET");
    assert!(
        matches!(
            target.as_str(),
            "x86_64-pc-windows-msvc" | "aarch64-pc-windows-msvc"
        ),
        "Only Windows x64 and ARM64 MSVC targets are supported"
    );
    // No DLL entry point or CRT initialization is needed: this addon uses only core and Win32.
    println!("cargo:rustc-link-arg=/NOENTRY");
    println!("cargo:rerun-if-changed=build.rs");
}
