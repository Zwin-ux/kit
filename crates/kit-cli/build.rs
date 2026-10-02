//! The starter kits in `kits/` are embedded with `include_dir!`, which cargo
//! cannot see. Rebuild when any file under `kits/` changes.
fn main() {
    println!("cargo:rerun-if-changed=kits");
    println!("cargo:rerun-if-changed=claude-plugin/hooks");
    println!("cargo:rerun-if-changed=claude-plugin/agents");
    println!("cargo:rerun-if-changed=claude-plugin/skills");
    println!("cargo:rerun-if-changed=claude-plugin/themes");
    println!("cargo:rerun-if-changed=claude-plugin/.claude-plugin/plugin.json");
    println!("cargo:rerun-if-changed=claude-plugin/README.md");
    println!("cargo:rerun-if-changed=claude-plugin/LICENSE");
    println!("cargo:rerun-if-changed=claude-plugin/provenance.json");
}
