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
    // Save section headers/alignment padding by merging read-only unwind data into .rdata.
    // The PE exception directory preserves unwinding, but tools that require a named .pdata
    // section may fail, and its section attributes can no longer be controlled separately.
    println!("cargo:rustc-link-arg=/MERGE:.pdata=.rdata");
    println!("cargo:rerun-if-changed=build.rs");
}
