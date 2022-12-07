fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=makefile");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=amd64.json");
    println!("cargo:rerun-if-changed=.cargo/**");
    println!("cargo:rerun-if-changed=link.ld");
}
