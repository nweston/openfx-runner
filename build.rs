fn main() {
    println!("cargo:rerun-if-changed=src/params.c");
    cc::Build::new().file("src/params.c").compile("params");
}
