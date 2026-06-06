default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace --locked

doc:
    cargo doc --workspace --no-deps --locked

scaffold-check:
    bash scripts/check-mobile-scaffold.sh

check: fmt test scaffold-check

ci: fmt-check lint test doc scaffold-check
