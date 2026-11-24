# Zenith testing

## Test runner and commands

There is **no `test` script** in [`package.json`](../package.json). Its only
scripts are `dev`, `build`, `preview`, and `tauri`. Automated tests are Rust
`#[test]` functions in the two Cargo manifests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/worker/Cargo.toml
```

These are standard Cargo invocations for the manifests in this repository; the
project does not define a custom test wrapper. `npm run build` runs `tsc &&
vite build`, which is the only type-checking step declared in the manifest.
No lint script or dependency-free frontend test runner is configured.

## Test inventory

| File | Tests declared in source | Focus |
| --- | --- | --- |
| [`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs) | 6 | `merge_temps`: hardware/mock source selection, ACPI fallback, non-finite filtering, and `none` with/without a worker error. |
| [`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs) | 1 (`#[cfg(windows)]`) | Spawns the built worker and waits for a healthy mock sample. It sets `ZENITH_WORKER_PATH`/`ZENITH_WORKER_MOCK` and requires a release worker binary first. |
| [`src-tauri/worker/src/protocol.rs`](../src-tauri/worker/src/protocol.rs) | 4 | JSONL round-trips for sample and error envelopes, error-code serialization names, and `Quality` serialization. |
| [`src-tauri/worker/src/intel.rs`](../src-tauri/worker/src/intel.rs) | 5 | Raw-MSR-to-temperature decode fixtures, TjMax detection, and quality classification. |
| [`src-tauri/worker/src/mock.rs`](../src-tauri/worker/src/mock.rs) | 3 | Mock backend samples and unavailable state. |
| [`src-tauri/worker/src/supervisor.rs`](../src-tauri/worker/src/supervisor.rs) | 7 | Circuit-breaker backoff, failure/healthy transitions, and state reset. |
| [`src-tauri/worker/src/topology.rs`](../src-tauri/worker/src/topology.rs) | 2 | CPU topology handling fixtures. |
| [`src-tauri/worker/tests/cli.rs`](../src-tauri/worker/tests/cli.rs) | 2 | Spawns the worker binary: `--mock` emits healthy sample envelopes; `--mock --unavailable` emits an error and exits with code 1. |

The counts above come from the `#[test]` attributes currently in the tree. The
[`CHANGELOG.md`](../CHANGELOG.md) 1.1.0 entry records 23 worker tests (21 unit
plus 2 CLI integration), which matches the worker files above.

## What is genuinely covered

- Pure temperature-source merging, including the requirement that a labeled
  fallback is never presented as hardware data.
- Worker protocol serialization at version 1, Intel MSR decode against raw
  fixtures, mock backend behavior, topology, and circuit-breaker state.
- Worker CLI output for the mock and unavailable paths, using
  `CARGO_BIN_EXE_zenith-sensor-worker`.

## Gaps

- No frontend unit, component, or end-to-end tests were found; there is no
  Vitest, Jest, or Playwright configuration.
- No coverage report is committed, so no coverage percentage can be reported.
- No CI workflow for Zenith itself was found. The workflows under `dev/lhm` and
  `dev/pawnio-modules` belong to ignored vendored reference repositories.
- Live elevated hardware validation is described in [`CHANGELOG.md`](../CHANGELOG.md)
  and [`dev/release/1.1.0/RELEASE_NOTES.md`](../dev/release/1.1.0/RELEASE_NOTES.md),
  but it is not an automated test in this repository.
- The supervisor streaming test is Windows-only and expects the worker to have
  been built first; other platform backends are selected at compile time and
  have no automated end-to-end test here.

## Sources

[`package.json`](../package.json), [`CHANGELOG.md`](../CHANGELOG.md),
[`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs),
[`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs),
[`src-tauri/worker/src/intel.rs`](../src-tauri/worker/src/intel.rs),
[`src-tauri/worker/src/mock.rs`](../src-tauri/worker/src/mock.rs),
[`src-tauri/worker/src/protocol.rs`](../src-tauri/worker/src/protocol.rs),
[`src-tauri/worker/src/supervisor.rs`](../src-tauri/worker/src/supervisor.rs),
[`src-tauri/worker/src/topology.rs`](../src-tauri/worker/src/topology.rs),
[`src-tauri/worker/tests/cli.rs`](../src-tauri/worker/tests/cli.rs).
