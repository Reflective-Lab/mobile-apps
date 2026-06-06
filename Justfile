default:
    @just --list

fmt:
    cargo fmt --all

test:
    cargo test --workspace

check: fmt test

