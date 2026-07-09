use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/gui.ts");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let source = manifest_dir.join("ui").join("gui.ts");
    let compiled = out_dir.join("gui.js");
    let embedded = out_dir.join("embedded_gui.rs");

    let status = Command::new("tsc")
        .arg("--target")
        .arg("ES2022")
        .arg("--module")
        .arg("none")
        .arg("--lib")
        .arg("ES2022")
        .arg("--outFile")
        .arg(&compiled)
        .arg(&source)
        .status();

    match status {
        Ok(status) if status.success() && compiled.exists() => {}
        Ok(status) => panic!("tsc emit failed with status: {status}"),
        Err(error) => panic!("failed to run tsc emit step: {error}"),
    }

    std::fs::write(
        embedded,
        format!("pub const EMBEDDED_GUI_JS: &str = include_str!(concat!(env!(\"OUT_DIR\"), \"/gui.js\"));\n"),
    )
    .expect("failed to write embedded_gui.rs");
}
