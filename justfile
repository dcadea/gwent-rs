# Run tests with coverage
cov:
    cargo llvm-cov \
        --open

clippy:
    cargo fmt
    cargo clippy --all-features --all-targets -- \
        -W clippy::pedantic \
        -W clippy::nursery \
        -W clippy::unwrap_used
