# `just` runner — `cargo install just`, then `just <recipe>`.
# Mirrors what CI runs so a green local run means a green CI run.

default: build

# Build all binaries + library in release mode.
build:
    cargo build --release --all-targets

# Run the whole test suite.
test:
    cargo test --all-targets

# Lint: clippy with warnings as errors + fmt check.
lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

# RustSec advisory check. Requires `cargo install cargo-audit`.
audit:
    cargo audit

# CI parity: everything CI runs, in CI's order.
ci: lint test build audit

# Render the popup PNGs into assets/screenshots/ (dark + light).
# Used after touching src/bin/popup/render.rs to keep README screenshots fresh.
screenshots:
    cargo build --release --bin podctl-popup
    ./target/release/podctl-popup --dump assets/screenshots/popup-dark.png --theme dark
    ./target/release/podctl-popup --dump assets/screenshots/popup-light.png --theme light

# Wipe the build cache.
clean:
    cargo clean
