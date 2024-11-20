fn main() {
    println!("cargo::rustc-link-search=../libc");
    println!("cargo::rustc-link-lib=c");

    let bindings = bindgen::Builder::default()
        .header("../libc/sys/stat.h")
        .header("../libc/ctype.h")
        .header("../libc/stat.h")
        .header("../libc/stdio.h")
        .header("../libc/stdlib.h")
        .header("../libc/string.h")
        .header("../libc/syscalls.h")
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
