//! The starter kits in `kits/` are embedded with `include_dir!`, which cargo
//! cannot see. Rebuild when any file under `kits/` changes.
fn main() {
    println!("cargo:rerun-if-changed=kits");
}
