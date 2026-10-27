# Pinned beta integration

Target beta: `fd300e276356587726566026f072563eb9c34cb1`. Nothing in this repository edits the host checkout or a profile. The validation Tauri/router/registration/installation seams are test stubs, clearly labeled in `validation/`; they do not demonstrate a rebuilt production app.

1. Vendor this package at the stable host path `src-tauri/native-extensions/css-calculations`. Keep `Cargo.toml`, `Cargo.lock`, `src/`, `adapter/`, assets and notices together; avoid a temporary-clone path dependency.
2. Add the dependency to the host's `src-tauri/Cargo.toml`:

   ```toml
   css-calculations-native-v2 = { path = "native-extensions/css-calculations" }
   ```

3. In `src-tauri/src/native_core/extensions/mod.rs`, add this sibling of the existing `v2_sdk`:

   ```rust
   #[path = "../../../native-extensions/css-calculations/adapter/css_calculations.rs"]
   mod v2_css_calculations;
   ```

4. Update `v2_runtime.rs`: recognize `css-calculations` in `slug` (including `raycast.css-calculations` and `com.tezbar.native.css-calculations` aliases), return `super::v2_css_calculations::commands()` in `commands`, and dispatch the slug through:

   ```rust
   super::v2_css_calculations::handle(ctx, command, operation, payload, state).await
   ```

   Both commands must match exactly: `clamp_px`, `clamp_rem`. Keep command/platform coverage assertions connected to actual compiled dispatch. Add the manifest to the host `v2_catalog.json` in the format the catalog expects, asserting canonical/replacement IDs, exact command coverage and both platform declarations. The catalog must not claim manifest-only execution or a QA pass.
5. Copy the byte-exact `assets/css.png` to **`src/renderer/public/extensions/icons/css-calculations.png`**, serving `/extensions/icons/css-calculations.png`; verify SHA256 `d05434a703c9ac222778979ec8d451ebf439456520d4d2aa2c7ef04be770978f` (223702 bytes).
6. Register/bundle the canonical schema-2 manifest, rebuild the native binary, install through the actual native extension registry, and verify listing/install state through the compiled coverage CLI. Leave V1 installed for fallback. No legacy storage migration is claimed. Real install, host registration/rebuild, UI/clipboard/focus/paste QA must be completed separately on each OS before changing its `not-run` verification.

## Runtime contract

The adapter directly imports the unchanged SDK and `CoreContext`. It receives actual launch `payload.argumentValues` and `payload.preferences`; neither is declared by this source, so neither is used. All operation names include `extension:`: `run-command`, `invoke-action`, `search-text-changed`, `refresh-session`, `load-more`, `dispose-session`.

The adapter emits supported `Form` / `Form.TextField` / `Form.Description` nodes, actions with `kind: submit-form` or `action`, field callbacks through `formValues.value`, and complete submit maps. Field IDs and action bindings stay stable per session. Only the changed controlled field returns a value; unrelated fields omit it so newer queued renderer drafts survive. Refresh/search/load-more return `unchanged` without new bindings or mutations. There is no server-search list or additional navigation stack. Back exits the source form; in-session draft values are retained on rejected events and failed effects. Reopening starts source defaults.

Bindings use a process-wide monotonic nonce with explicit exhaustion error, never a counter that can roll back with the host's staged search clone. Malformed requests leave legitimate bindings intact. IDs from another session, disposed sessions, old launches or discarded state are rejected. NativeSessions serializes actions and handles installed checks/cancellation. Copy intent is written before awaiting the router; cancellation leaves a pending/uncertain marker, and only deliberate retry repeats that output. Capabilities are internal routing bindings, not authentication credentials.

The only effect used is SDK `copy` → `clipboard:write` with `[computed_text]`. Success requires an object with `ok:true`, `error` absent/null, and `success` absent/true. Null, booleans, contradictory flags, malformed responses and transport errors never become success. Untrusted router error details are not reflected into the UI. No clipboard reads, selection, paste, hide, browser or focus route is invoked. In the pinned beta, `clipboard:read []` and `core:platform:selected-text []` return strings; `core:platform:paste-text [text]` already writes/hides/restores focus/waits/pastes; `core:platform:paste []` uses prepared clipboard; `window:hide []` performs hide but returns null; SDK open uses `shell:open [target]`. These routes must not be substituted into this command's copy-only behavior.

No storage is needed or used. If the source later adds persistence, use SDK `update_store` for process-atomic changes, preserve unknown keys and reject corrupt JSON/malformed known settings inside the transaction before overwriting. Do not infer legacy keys. No arbitrary shortcut metadata is advertised.
