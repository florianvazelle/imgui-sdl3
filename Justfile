default:
    just --list

lint:
    cargo fmt --check
    cargo check --all --workspace --examples --tests
    cargo clippy --all --workspace --examples --tests -- --deny warnings

fmt:
    cargo fmt
    cargo fix --all --workspace --examples --tests --allow-dirty
    cargo clippy --all --workspace --fix --examples --tests --allow-dirty -- --deny warnings

test:
    cargo run --example demo