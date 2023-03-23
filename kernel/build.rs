fn main() {
    println!("cargo:rerun-if-changed=amd64.ld");
    println!("cargo:rerun-if-changed=targets");
}
