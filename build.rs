use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/gui.ts");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let source = manifest_dir.join("ui").join("gui.ts");
    let compiled = manifest_dir.join("ui").join("gui.js");
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
        Ok(status) => panic!("tsc failed with status: {status}"),
        Err(error) => panic!("failed to run tsc: {error}"),
    }

    let gui_js = std::fs::read_to_string(&compiled).expect("failed to read compiled gui.js");
    std::fs::write(
        embedded,
        format!("pub const EMBEDDED_GUI_JS: &str = r###\"{}\"###;\n", gui_js),
    )
    .expect("failed to write embedded_gui.rs");
}
