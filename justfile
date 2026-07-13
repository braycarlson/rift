build:
    cargo build

check:
    cargo clippy --all-targets

srcgen:
    cargo run --package srcgen
    cargo fmt

srcgen-offline:
    cargo run --package srcgen -- --offline
    cargo fmt
