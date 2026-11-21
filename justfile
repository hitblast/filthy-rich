clippy:
    cargo clippy --all-targets --no-deps -- -D warnings
dirtyfix:
    cargo fix --all-targets --allow-dirty
fmt:
    cargo fmt --all
fmt-check:
    cargo fmt --all -- --check

testall: clippy fmt-check
fixall: fmt dirtyfix
