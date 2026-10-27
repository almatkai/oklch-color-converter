# Integration into the compiled Native V2 beta

These are review instructions and separate patches; they were not applied to the host. Baseline: host Git object `fd300e276356587726566026f072563eb9c34cb1`, not `origin/main`. The host workingtree is unrelated and dirty.

1. Vendor the contents of this `v2/` crate at **`src-tauri/native-extensions/oklch-color-converter`**. A Cargo package extraction is preferable because it excludes build artifacts while retaining the complete offline fixtures and harness. Keep the licenses/notices and Cargo.lock. The host consumes the crate as a path dependency; do not add a plugin loader or JS fallback.
2. Review `registration.patch`: add `tezbar-oklch-color-converter = { path = "native-extensions/oklch-color-converter" }` to the host dependencies; register the adapter from `src-tauri/src/native_core/extensions/mod.rs` using:

   ```rust
   #[path = "../../../native-extensions/oklch-color-converter/adapter/oklch.rs"]
   mod v2_oklch;
   ```

   The adapter imports the sibling `super::v2_sdk` and the real `CoreContext`. The existing host serde, serde_json and uuid/v4 dependencies supply adapter support. Update the host Cargo.lock when integrating.
3. Add `oklch-color-converter` to `v2_runtime::slug`, route `commands()` to `v2_oklch::commands()`, and route `handle()` to `v2_oklch::handle(ctx, command, operation, payload, state).await`. The concrete handler accepts `extension:run-command`, `extension:invoke-action`, `extension:search-text-changed`, `extension:refresh-session`, `extension:load-more` and `extension:dispose-session`.
4. Append `extension-v2.json` to `src-tauri/src/native_core/extensions/v2_catalog.json`. Keep canonical ID, replacement ID, the one command, and both `not-run` verification entries. The schema has no root `author`; credit strongeron in the description/notices. Update catalog count assertions (the pinned 11 becomes 12; after other sequential ports, calculate the actual total). Do not overwrite catalog entries from earlier ports.
5. Copy **`assets/icon.png`** byte-for-byte to **`src/renderer/public/extensions/icons/oklch-color-converter.png`**. Manifest URL: `/extensions/icons/oklch-color-converter.png`. Size: 202561; SHA256: `8bcde32c4dc7b94a8e119d3576153f34aee95bd416f937b2b93c8b56471af1e9`.
6. Review and apply the separate **required** `server-search.patch`. Its single condition prevents the generic color-converter override when a real server-search handler owns the list. Without it, valid searches can display unrelated rows and bypass the adapter's capability-bound actions. `scripts/renderer-contract.cjs` reproduces the original bug and verifies the fix in memory against the actual pinned TSX.
7. Run the host native session/catalog tests and update command/count tests for the new entry. Rebuild the host and inspect the compiled CLI's coverage, then install the canonical manifest through the host's ordinary Native V2 install path. A marker or manifest alone does not execute code.
8. QA on real macOS and Windows: launch empty, type valid/invalid/alpha/P3 colors rapidly, select all eight rows, inspect swatches/tags, run both copy actions, verify uncertain-result retry feedback, and close/reopen sessions. Check real color profiles and OS clipboard behavior. Record evidence before changing verification states.

The registration patch is exact against the pinned baseline. Reconcile its small hunks with previous sequential ports in the selected host branch instead of applying it blindly to a later catalog. The harness uses the same runtime registrations and verifies canonical, `raycast.oklch-color-converter` and bare-slug dispatch. `com.tezbar.oklch-color-converter` is the own V1 package ID and stays on the compatibility path; it is not falsely claimed as a Native V2 alias.

Keep the V1 package and its installed data intact. This port writes no storage and guesses no migration. The beta schema's replacement entry remains `raycast.oklch-color-converter` as required by the Recommended catalog. No global service, host profile, clipboard, focus or paste changes are part of integration preparation.
