default:
    @just --list

fmt:
    nix fmt

watch $RUST_BACKTRACE="1":
    cargo leptos watch

watch-release:
    cargo leptos watch --release

test:
    cargo watch -- cargo leptos test
