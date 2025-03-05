use std::path::PathBuf;

fn main() {
    // Get the path to the Alumet FFI bindings.
    // The bindings are generated by `alumet-ffi` build script.
    // The path to the bindings is not stable across builds, but it is given by `alumet-ffi` build script as a cargo::metadata,
    // which we can read through environment variables.
    let bindgen_out_dir = std::env::var("DEP_ALUMET_H_BINDINGS_DIR")
        .expect("cargo metadata BINDINGS_DIR should be set for 'links' alumet_h");
    println!("bindgen_out_dir: {bindgen_out_dir:?}");

    // Path the path to the crate's code
    println!("cargo:rustc-env=ALUMET_H_BINDINGS_DIR={}", bindgen_out_dir);

    let bindgen_out_dir = PathBuf::from(bindgen_out_dir);
    let alumet_ffi_symbols = bindgen_out_dir.join("alumet-symbols.txt");
    let symfile_path = alumet_ffi_symbols.canonicalize().unwrap();

    // Add link flags
    let linker_flags = format!("-Wl,--dynamic-list={}", symfile_path.to_str().unwrap());
    println!("cargo:rustc-link-arg={}", linker_flags);
}
