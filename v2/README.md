# OKLCH Color Converter — Native V2

Offline Rust library and a concrete compiled-in adapter for Tezbar's Native V2 beta. Original extension by **strongeron**; the own V1 source is pinned at `cc5022de97787bf2b9bf8f09be41242aa73f98fb`. Its lock resolves **Culori 3.3.0**, not the minimum `^3.2.0` in the manifest. V1 remains available unchanged.

The one command is `convert-color` / **Convert Color**. Native V2 ID: `com.tezbar.native.oklch-color-converter`; replaces `raycast.oklch-color-converter`. This is a library linked into the host Rust binary, with no external plugin loader or JavaScript runtime fallback. JavaScript under `fixtures/` and `scripts/` is test-only.

The list preserves the original order: Figma P3, OKLCH, Display P3, OKLAB, vec, hex, RGB, HSL. Every row has its original sRGB preview swatch, gamut tag, **Copy Color Value** and **Copy as Hex**. The internal `hex/rgba` and `lrgb` helper formats are also implemented; the source does not render separate rows for them. Bare eight-digit `#RRGGBBAA` is Figma P3, whereas the same digits without `#` use Culori's sRGB parser. Invalid Figma P3 prefixes become black, as in V1.

The port uses the source's conversion graph, XYZ65 proxy round trip, P3 upper bound of 1.6, byte clamping, alpha quirks, signed transfer functions, double linearization, hardcoded vec reference neighborhoods, and JavaScript decimal/padding rules. It does not use a generic replacement color converter. CSS modern/legacy syntax, all 148 named colors, and all 29 locked Culori modes (including custom `color(--…)` profiles) are covered. CSS `none` remains distinct from a computed NaN. V1 does not parse its emitted `vec(...)` strings; this limitation is preserved.

Read [integration/README.md](integration/README.md) before host integration. **The supplied server-search renderer patch is required**: the pinned renderer otherwise replaces the adapter's rows with its unrelated generic converter. No host changes have been applied here.

## Reproduce

From this directory, after the Cargo dependencies are cached:

```sh
./scripts/validate.sh
cargo +1.85.0 test --locked --offline
python3 scripts/beta-harness.py +1.85.0 test --locked --offline
cargo check --locked --offline --all-targets --target x86_64-pc-windows-gnu
cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-gnu -- -D warnings
python3 scripts/beta-harness.py check --locked --offline --all-targets --target x86_64-pc-windows-gnu
python3 scripts/beta-harness.py clippy --locked --offline --all-targets --target x86_64-pc-windows-gnu -- -D warnings
```

Packaging was checked with `cargo package --allow-dirty --locked --offline`. To repeat the extracted test, unpack `target/package/tezbar-oklch-color-converter-0.1.0.crate` into a disposable directory, enter its `tezbar-oklch-color-converter-0.1.0` directory, and run `sh scripts/validate.sh`. It has no dependency on the host checkout or the original clone path.

On a new machine, first run `cargo fetch --locked` and `python3 scripts/beta-harness.py fetch --locked`. Node and Python are needed for the test harnesses, not production. The included pure JS Culori source and Ajv schema validator run offline, without npm installation. No test reads the real clipboard, profile, selected text or desktop focus.

`node scripts/oracle.cjs` executes the included generated TSX code with exact locked Culori. To regenerate that code independently, create a disposable checkout of the pinned V1 fixture, run `npm ci --legacy-peer-deps` using its unchanged lock, then pass its `node_modules` path to `node scripts/oracle.cjs <path>` and `node scripts/renderer-contract.cjs --compile <path>`. The latter also needs a fresh adapter snapshot: `python3 scripts/beta-harness.py run --locked --quiet --example view > validation-output/view.json`. `scripts/generate-inputs.py` reproduces the deterministic corpus including source-generated round trips. `scripts/nonfinite.py` records the original behavior with bounded child processes.

The beta harness compiles the **real adapter**, unchanged pinned SDK/common/NativeSessions files, and a registration-only modification of the pinned runtime. Only Tauri event emission, installation policy and unrelated adapters are stubs. Its manifest is a template materialized for one Cargo invocation: Cargo otherwise silently excludes nested packages from `.crate` archives. Do not run two harness invocations concurrently in the same tree.

## Known differences and limits

- The oracle has **897 inputs**. **878 match all ColorData fields exactly** on the tested aarch64 macOS toolchains. The remaining **19 inputs have 22 explicit formatted-value exceptions** in `fixtures/oracle/numeric-deviations.json`: HSL hue near neutral/wrap boundaries and last digits at extremely large finite magnitudes. Every parsing decision, row order, tag, preview hex, and hex-copy value matches this corpus. The exceptions are tested as exact reviewed pairs, not hidden behind a tolerance. This is not a proof of universal numerical equivalence or real Windows parity.
- Non-finite input numbers receive an explicit safe error rather than V1's `NaN`/`Infinity` display artifacts. Finite input is bounded to 65,536 UTF-8 bytes; output is bounded to 4,096 bytes per value, at most eight rows. No truncation or unbounded cache is used. Conversion work has a fixed graph and no gamut-search loops.
- The beta action renderer does not dispatch arbitrary per-action shortcuts. V1's Cmd+C / Cmd+Shift+C are recorded in the oracle but are **not advertised as functional shortcuts** in the adapter. Enter/click runs the selected row's first action; the action menu exposes both copy choices. The pinned menu opener uses Meta+K; Control+K support on Windows is not claimed.
- V1 failure toasts are not a beta API. Copy success/uncertainty uses supported view messages and explicit Retry action labels. Invalid parser exceptions clear the list; V1's additional parse-error toast is not reproduced. Neither HUD nor automatic window hiding/pop-to-root is invented. No `window:hide`, Markdown code-copy, paste or focus helper is used, so no acknowledgement or Markdown prerequisite patch is needed.
- The adapter deliberately omits controlled `searchText` in returned views. The real beta renderer keeps the user's local draft; a delayed refresh/action response cannot overwrite a newer queued search. Actual server search uses `__hasServerSearch: true` and `filtering: false`.
- Copy output comes only from canonical committed rows. Intent and a fresh capability are saved before awaiting SDK copy. A positive, noncontradictory object acknowledgement is required. Null, booleans, malformed, contradictory and transport replies produce an uncertain result, never success. No action is retried automatically. Refresh is read-only and returns existing committed IDs. Process UUID + monotonic tokens prevent discarded search clones from reviving capabilities. Acknowledged stages survive later failures.
- No storage, preferences, launch arguments or clipboard initialization are invented: the V1 command uses none of these.

See [VALIDATION.md](VALIDATION.md), [NOTICES.md](NOTICES.md) and [provenance.json](provenance.json). **Real macOS/Windows OS/UI/color-profile/clipboard/focus QA, host registration and host rebuild remain NOT-RUN.** Both manifest verification entries remain `not-run`.
