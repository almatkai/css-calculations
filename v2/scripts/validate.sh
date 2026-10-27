#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .validation
python3 scripts/materialize.py
for toolchain in stable 1.85.0; do
  cargo +"$toolchain" fmt -- --check
  rustup run "$toolchain" rustfmt --edition 2021 --check --config skip_children=true adapter/css_calculations.rs validation/host/lib.rs.in validation/host/contract.rs.in validation/tauri/lib.rs.in
  cargo +"$toolchain" test --locked --offline
  cargo +"$toolchain" clippy --locked --offline --all-targets -- -D warnings
  CSS_WIRE_OUTPUT="$PWD/.validation/wire.json" cargo +"$toolchain" test --manifest-path .validation/host/Cargo.toml --locked --offline
  cargo +"$toolchain" clippy --manifest-path .validation/host/Cargo.toml --locked --offline --all-targets -- -D warnings
done
for manifest in Cargo.toml .validation/host/Cargo.toml; do
  cargo +stable check --manifest-path "$manifest" --locked --offline --all-targets --target x86_64-pc-windows-gnu
  cargo +stable clippy --manifest-path "$manifest" --locked --offline --all-targets --target x86_64-pc-windows-gnu -- -D warnings
done
cargo +stable build --locked --offline --bin css-oracle-driver
node scripts/oracle.cjs
node scripts/renderer.cjs
.validation/python/bin/python scripts/verify.py
npm test --prefix fixtures/own
npm run typecheck --prefix fixtures/own
