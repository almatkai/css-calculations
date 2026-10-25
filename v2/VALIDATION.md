# Reproducible validation

Run from `v2/`. Rust **1.85.0** and stable **1.98.0** were checked on aarch64 macOS. Install the corresponding Clippy/rustfmt components and stable `x86_64-pc-windows-gnu` target before checking. Node used here: **26.7.0**. Python requires `jsonschema==4.23.0` (full Draft 2020-12 validation, not a hand-rolled subset).

Initial dependency preparation may use the network. Runtime and all Cargo checks below are offline after preparation:

```sh
mkdir -p .validation
python3 scripts/materialize.py
cargo +stable fetch --locked
cargo +stable fetch --manifest-path .validation/host/Cargo.toml --locked
npm ci --ignore-scripts --prefix fixtures/own
python3 -m venv .validation/python
.validation/python/bin/pip install 'jsonschema==4.23.0'
bash scripts/validate.sh
python3 scripts/package-check.py --node
```

`materialize.py` writes disposable harness crates below `.validation/` from `validation/*/*.in`. Cargo omits nested crates with real manifests during packaging, so the shipped templates include manifests, the host lock, source and integration tests. Absolute fixture paths are materialized for the current extraction, not stored as developer-machine dependencies. The harness depends on the engine through `../..`. `.validation`, targets and `node_modules` are excluded from both Git and Cargo.

## Results

- Engine tests: **1**. Actual beta common/SDK unit tests: **5**. Adapter/actual NativeSessions contract tests: **9**. All pass on stable and Rust 1.85, offline. The SDK tests include process-atomic concurrent storage updates, unknown-key preservation, corrupt JSON refusal and routing failures. The adapter uses no storage because neither source command persists anything.
- Strict formatting and `--all-targets` Clippy with `-D warnings`: pass on both Rust toolchains for engine and adapter harness. Template/adapter formatting uses `skip_children=true` so the unchanged beta fixtures are not rewritten.
- Stable Windows GNU `check --all-targets` and strict Clippy: pass for engine and production adapter with unchanged beta common/SDK/NativeSessions fixtures. This is cross compilation with a Tauri stub, not Windows execution or a full host build.
- Executable source oracle: **16,703** deterministic cases comparing exact CSS bytes, `validateAllValues` and on-change `Number` validation. It executes actual pinned `utils.ts`, not a re-created JS formula. Both actual TSX commands and actual hooks execute against locked React to confirm defaults, string-zero/change/submit differences, mismatched error keys and copy → HUD order. Cases include random binary64 bit patterns, dense rounding boundaries, decimals, exponent notation, subnormals, `NaN`/Infinity, null/booleans/arrays/objects, invalid/radix/Unicode strings, empty/whitespace/negative/zero values and equal/reversed/near-equal viewports. No claim of exhaustive enumeration of all f64 values is made. JavaScript `Number` and `toFixed` differ from parseFloat input parsing; only the source's final `parseFloat(toFixed(4))` is reproduced.
- Actual pinned renderer `collectFormItems`, `reconcileFormValues`, `ExtensionRuntimeSurface` and `FormRuntime` handlers execute against adapter-produced wire responses. They check full-map submission, `formValues.value` callbacks, stable IDs and preservation of unrelated queued drafts. React's dispatcher/DOM effects are test seams; no browser window or OS effect is opened.
- Effect/lifecycle tests cover strict acknowledgement positives and malformed/null/boolean/contradictory/transport failures, sanitized errors, deliberate retry bound to prior output, interrupted copy checkpoint, forged/cross-session/stale IDs, malformed-event binding preservation, discarded clone/non-revival, unchanged refresh/search, uninstall/dispose/clear and concurrent submissions. Search is intentionally a no-op for these forms. Beta's original `native_tests.rs` is retained byte-exact for provenance, but its unrelated full-host/Perplexity suite is not executed; the external contract suite runs the actual unmodified NativeSessions implementation.
- Pinned full schema validates; **21** invalid manifests are rejected. Additional assertions compare exact command metadata with own source, canonical/replacement IDs, no root author, preferences/arguments and both OS `not-run` records.
- **180** provenance paths pass byte-count/SHA256 and Git visibility checks, including **121** actual Rust license/notice texts, the own lock/source and original 223702-byte PNG. All existing tracked V1 files except the README discovery link remain byte-exact; the Git index is empty.
- Cargo package verification and self-contained extracted offline engine/adapter tests + strict Clippy pass. Every provenance path and harness template is in the archive. The extracted package independently repeats the Node oracle, renderer contract, original V1 smoke test and TypeScript typecheck using an offline npm cache. Extracted checks use only packaged source/fixtures/templates plus pre-fetched Cargo/npm dependencies and the installed Python schema validator.
- Disposable V1 fixture: original smoke test **1 pass** and original `tsc --noEmit` pass; lock unchanged. npm audit observed 3 validation dependency vulnerabilities (2 low, 1 high), documented without modifying the source lock.

## Limits and remaining QA

XERJ used canonical root prefix `ax-f49cadc61662` and isolated state. Initial index completed with all **6/6** code files indexed, 12 content files / 90 records; binary/hidden exclusions are not evidence of coverage. Exact `rg` and Git/tree/hash checks cover the remaining artifacts. The final reindex aborted with the existing frozen JSON dataset/schema evolution error when it reached the new schema fixture. Final V2 XERJ coverage is therefore incomplete; no shared schema or service was changed. Exact Git/provenance checks and `rg` were used for the final audit.

Fields above **65,536 UTF-8 bytes** are rejected explicitly as a resource error, never truncated. Within that bound the form adds no numeric constraints. JSON numeric input is parsed with `serde_json/float_roundtrip` so binary64 oracle values match JavaScript; real form transport accepts source-compatible strings only.

HUD adaptation is a persistent form description/output after acknowledged copy, with drafts retained; transient HUD and window dismissal are not implemented. The pinned host's null `window:hide` response is not accepted as success. Any future dismissal integration requires a host acknowledgement fix and separate QA. There is no fake toast/HUD/pop API or unsupported shortcut claim.

Host registration/rebuild, real installation, actual macOS and Windows UI/clipboard/selection/paste/browser/window/focus QA are **not-run**. No actual OS clipboard, selection, paste, focus, browser or window was touched by tests. Both manifest verification states remain **not-run**. Nothing is staged, committed, pushed, deployed or deleted by this implementation task.
