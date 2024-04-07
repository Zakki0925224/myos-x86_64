fn main() {
    println!("cargo::rustc-link-search=../libm");
    println!("cargo::rustc-link-lib=m");

    let bindings = bindgen::Builder::default()
        .header("../libm/libm.h")
        .use_core()
        .generate()
        .expect("Failed to generate bindings");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Fialed to wrote bindings");
}
