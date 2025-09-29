use std::path::Path;

fn main() {
    // Tell cargo to re-run build if the terminal-notifier.app changes
    println!("cargo:rerun-if-changed=resources/terminal-notifier.app");

    // Verify that terminal-notifier.app exists at build time
    let app_path = Path::new("resources/terminal-notifier.app");
    if !app_path.exists() {
        panic!("terminal-notifier.app not found in resources directory. Please ensure it exists.");
    }
}