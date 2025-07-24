default:
    just --list

fmt:
    cargo fmt
    cargo fix --workspace --examples --allow-dirty
    cargo clippy --workspace --fix --examples --allow-dirty -- -D warnings

test:
    cargo run --example demo