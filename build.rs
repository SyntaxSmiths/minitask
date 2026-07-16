#[cfg(feature = "gui")]
use std::path::PathBuf;
#[cfg(feature = "gui")]
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    #[cfg(feature = "gui")]
    {
        println!("cargo:rerun-if-changed=ui/gui.ts");
        println!("cargo:rerun-if-changed=ui/tsconfig.json");
        println!("cargo:rerun-if-changed=generated/");

        let manifest_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
        let ui_dir = manifest_dir.join("ui");
        let generated_dir = manifest_dir.join("generated");

        // Create generated directory if it doesn't exist
        std::fs::create_dir_all(&generated_dir).expect("failed to create generated directory");

        // Run tsc with project config - outputs to generated/gui.js
        let status = Command::new("tsc")
            .arg("--project")
            .arg(&ui_dir)
            .current_dir(&manifest_dir)
            .status();

        match status {
            Ok(status) if status.success() => {
                // TypeScript outputs to generated/gui.js
                let compiled = generated_dir.join("gui.js");
                if !compiled.exists() {
                    panic!(
                        "tsc succeeded but gui.js not found at {}",
                        compiled.display()
                    );
                }
            }
            Ok(status) => panic!("tsc emit failed with status: {status}"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                panic!(
                    "TypeScript compiler (tsc) not found in PATH.\n\
                     \n\
                     The 'gui' feature requires TypeScript to compile the GUI code.\n\
                     \n\
                     Install TypeScript using your preferred method:\n\
                     \n\
                     • Fedora/RHEL:  sudo dnf install typescript\n\
                     • Debian/Ubuntu: sudo apt install typescript\n\
                     • Arch Linux:   sudo pacman -S typescript\n\
                     • npm:          npm install -g typescript\n\
                     • npx:          Use 'npx tsc' (no installation needed)\n\
                     \n\
                     Or build without GUI support:\n\
                     cargo build --no-default-features\n"
                );
            }
            Err(error) => panic!("failed to run tsc: {error}"),
        }
    }
}
