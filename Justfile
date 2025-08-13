default:
    just --list

lint:
    cargo fmt --check
    cargo check --all-targets --examples --tests --all-features
    cargo clippy --all-targets --examples --tests --all-features -- --deny warnings

fmt:
    cargo fmt
    cargo fix --all-targets --examples --tests --allow-dirty --all-features
    cargo clippy --all-targets --fix --examples --tests --allow-dirty --all-features -- --deny warnings

build:
    cargo build --examples --all-targets --all-features