fn main() {
    println!("cargo::rustc-link-search=../libc");
    println!("cargo::rustc-link-lib=m");

    let bindings = bindgen::Builder::default()
        .header("../libc/stdio.h")
        .header("../libc/string.h")
        .header("../libc/syscalls.h")
        .header("../libc/temp.h")
        .header("../libc/utsname.h")
        .header("../libc/window.h")
        .use_core()
        .generate()
        .expect("Failed to generate bindings");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Fialed to wrote bindings");
}
