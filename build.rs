use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/gui.ts");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=LICENSE");

    // Export package metadata as environment variables for the TypeScript compilation
    println!("cargo:rustc-env=MINITASK_VERSION={}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=MINITASK_NAME={}", env!("CARGO_PKG_NAME"));
    println!("cargo:rustc-env=MINITASK_AUTHORS={}", env!("CARGO_PKG_AUTHORS"));
    println!("cargo:rustc-env=MINITASK_REPOSITORY={}", env!("CARGO_PKG_REPOSITORY"));
    println!("cargo:rustc-env=MINITASK_LICENSE={}", env!("CARGO_PKG_LICENSE"));
    println!("cargo:rustc-env=MINITASK_DESCRIPTION={}", env!("CARGO_PKG_DESCRIPTION"));

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let source = manifest_dir.join("ui").join("gui.ts");
    let compiled = out_dir.join("gui.js");
    let embedded = out_dir.join("embedded_gui.rs");

    // Create a metadata file that will be prepended to the compiled JavaScript
    let metadata_js = format!(
        "// Build-time metadata injected by build.rs\nconst MINITASK_VERSION = \"{}\";\nconst MINITASK_NAME = \"{}\";\nconst MINITASK_AUTHORS = \"{}\";\nconst MINITASK_REPOSITORY = \"{}\";\nconst MINITASK_LICENSE = \"{}\";\nconst MINITASK_DESCRIPTION = \"{}\";\n\n",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_REPOSITORY"),
        env!("CARGO_PKG_LICENSE"),
        env!("CARGO_PKG_DESCRIPTION"),
    );

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

    // Read the compiled JavaScript
    let compiled_js = std::fs::read_to_string(&compiled)
        .expect("failed to read compiled gui.js");

    // Extract shebang if present and inject metadata after it
    let final_js = if compiled_js.starts_with("#!") {
        if let Some(newline_pos) = compiled_js.find('\n') {
            let shebang = &compiled_js[..=newline_pos];
            let rest = &compiled_js[newline_pos + 1..];
            format!("{}{}{}", shebang, metadata_js, rest)
        } else {
            format!("{}{}", metadata_js, compiled_js)
        }
    } else {
        format!("{}{}", metadata_js, compiled_js)
    };
    
    std::fs::write(&compiled, final_js)
        .expect("failed to write metadata-injected gui.js");

    std::fs::write(
        embedded,
        format!("pub const EMBEDDED_GUI_JS: &str = include_str!(concat!(env!(\"OUT_DIR\"), \"/gui.js\"));\n"),
    )
    .expect("failed to write embedded_gui.rs");
}
