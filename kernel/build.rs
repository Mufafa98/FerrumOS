fn main() {
    // Tell cargo to pass the linker script to the linker..
    // println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rustc-link-arg=--script=linker.ld");
    // ..and to re-run if it changes.
    // println!("cargo:rerun-if-changed=linker.ld");
}
