use std::{env, path::PathBuf, process::Command};

fn main() {
    // CARGO_MANIFEST_DIR points to crate dire
    let workspace_root = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap()
        .join("..");

    let frontend_path = workspace_root.join("frontend");

    let target = workspace_root
        .canonicalize()
        .unwrap()
        .join("target")
        .join("frontend");

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(&frontend_path)
        .env("OUTPUT_DIR", &target)
        .status()
        .unwrap();

    if !status.success() {
        panic!("npm build failed")
    }

    println!(
        "cargo:rerun-if-changed={}/src",
        frontend_path.to_string_lossy()
    );
}
