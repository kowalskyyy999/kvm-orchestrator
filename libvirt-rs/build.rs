use std::env;
use std::path::PathBuf;

fn main() {
    let libvirt = pkg_config::Config::new()
        .atleast_version("7.0.0")
        .probe("libvirt")
        .expect("libvirt not found");

    // Rebuild if wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(
            libvirt
                .include_paths
                .iter()
                .map(|p| format!("-I{}", p.display())),
        )
        .allowlist_function("vir.*")
        .allowlist_type("vir.*")
        .allowlist_var("VIR_.*")
        .generate()
        .expect("Unable to generate bindings");

    // Write bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
