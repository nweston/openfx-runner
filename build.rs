fn main() {
    println!("cargo:rerun-if-changed=src/variadic_functions.c");
    cc::Build::new()
        .file("src/variadic_functions.c")
        .compile("params");
}
