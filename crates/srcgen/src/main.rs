mod fetch;
mod generate;
mod naming;
mod spec;

use std::path::Path;

#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const SPEC_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/spec");
const SPEC_MANUAL_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/spec-manual");
const TARGET_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../rift/src/generated");

fn main() -> anyhow::Result<()> {
    let spec_directory = Path::new(SPEC_DIRECTORY);
    let spec_manual_directory = Path::new(SPEC_MANUAL_DIRECTORY);
    let target_directory = Path::new(TARGET_DIRECTORY);
    let offline = offline_requested();

    if offline {
        println!("offline mode: skipping spec fetch");
    } else {
        fetch::spec_files_fetch(spec_directory)?;
    }

    assert!(
        spec_directory.is_dir(),
        "spec directory must exist: {SPEC_DIRECTORY}"
    );

    generate::generate_all(spec_directory, spec_manual_directory, target_directory)?;

    assert!(
        target_directory.join("mod.rs").is_file(),
        "generated mod.rs must exist"
    );

    Ok(())
}

fn offline_requested() -> bool {
    let flag = std::env::args().any(|argument| argument == "--offline");
    let variable = std::env::var_os("SRCGEN_OFFLINE").is_some();

    flag || variable
}
