# Zenith configuration

## Environment variables

| Variable | Where it is read | Default and behavior |
| --- | --- | --- |
| `TAURI_DEV_HOST` | [`vite.config.ts`](../vite.config.ts) via `process.env.TAURI_DEV_HOST` | Unset: dev server binds `127.0.0.1:5173` (`strictPort: true`) with no HMR override. Set: used as the dev host and as the WebSocket HMR host on port `5174`. |
| `ZENITH_WORKER_PATH` | [`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs), `resolve_worker_exe`, under `#[cfg(debug_assertions)]` only | No default value. In debug builds, a path that `is_file()` overrides worker lookup. Release builds never read it. |
| `ZENITH_WORKER_MOCK` | [`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs), `mock_override`, under `#[cfg(debug_assertions)]` only | Unset/false by default. Values `1` or case-insensitive `true` pass `--mock` to the worker in debug builds. The release implementation always returns `false`. |

No `.env` file is committed. [`.gitignore`](../.gitignore) excludes `.env`,
`.env.local`, and `.env.*.local`.

## Worker command-line flags

Defined in [`src-tauri/worker/src/main.rs`](../src-tauri/worker/src/main.rs):

| Flag | Behavior |
| --- | --- |
| `--mock` | Uses `MockBackend`, whose samples are labeled `MOCK` in the UI. |
| `--unavailable` | Used with `--mock` to simulate a driver error and exit with code 1. |
| `--debug` | Writes backend open/sample timing messages to stderr. |

The supervisor only passes `--mock` itself when `ZENITH_WORKER_MOCK` is enabled
in a debug build.

## Config files and constants

| File | Settings |
| --- | --- |
| [`package.json`](../package.json) | Name `zenith`, version `1.1.0`, `private: true`, ESM, npm scripts and dependency ranges. |
| [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json) | Identifier `com.zenith.app`; window titled `Zenith`, 320×550, decorations off, transparent, non-resizable; dev URL `http://127.0.0.1:5173`; frontend output `../dist`; CSP; bundle targets `all`. |
| [`src-tauri/tauri.windows.conf.json`](../src-tauri/tauri.windows.conf.json) | Maps `worker/target/release/zenith-sensor-worker.exe` to `zenith-sensor-worker.exe` beside the app. |
| [`src-tauri/tauri.linux.conf.json`](../src-tauri/tauri.linux.conf.json) | Maps `worker/target/release/zenith-sensor-worker` to `zenith-sensor-worker` in the resource directory. |
| [`src-tauri/capabilities/default.json`](../src-tauri/capabilities/default.json) | Main-window permissions: `core:default`, start dragging, minimize, close, and `opener:default`. |
| [`src-tauri/app.manifest`](../src-tauri/app.manifest) | Windows `requestedExecutionLevel` is `asInvoker`; PerMonitorV2 DPI awareness. Elevation happens through the OS prompt in `lib.rs`, not this manifest. |
| [`.cargo/config.toml`](../.cargo/config.toml) | Cargo `[build] jobs = 2`. |
| [`tsconfig.json`](../tsconfig.json), [`tsconfig.node.json`](../tsconfig.node.json) | Strict TypeScript with `noEmit`; separate config for `vite.config.ts`. |
| [`src-tauri/worker/Cargo.toml`](../src-tauri/worker/Cargo.toml) | `rust-version = "1.75"`; release profile uses `opt-level = 3`, `lto = true`, `strip = "symbols"`, `panic = "abort"`. |
| [`.gitignore`](../.gitignore), [`src-tauri/.gitignore`](../src-tauri/.gitignore) | Ignore build output, generated schemas, `dev/`, local env files, and worker scratch `*.txt` files. |

Runtime constants are code-defined rather than user settings: the app emits a
payload every 1000 ms in [`lib.rs`](../src-tauri/src/lib.rs), the worker samples
every 1 second in [`main.rs`](../src-tauri/worker/src/main.rs), the supervisor
uses a 3-second hang timeout and 250 ms / 1 s / 5 s backoff, and process data is
refreshed every fifth poll in [`sensor.rs`](../src-tauri/src/sensor.rs).

## Local storage and user settings

No `localStorage` keys, persisted preferences, or settings screen were found.
The webview holds only the latest `system-metrics` payload in React state in
[`App.tsx`](../src/App.tsx). Thermal colors and insight thresholds are
hard-coded in [`Dashboard.tsx`](../src/components/Dashboard.tsx), for example a
package temperature above 90 °C produces `THROTTLING RISK`.

## Security notes

- No API keys, tokens, passwords, or other secrets were found in the tracked
  configuration or source. Do not commit local `.env` files.
- `ZENITH_WORKER_PATH` and `ZENITH_WORKER_MOCK` are deliberately debug-only:
  the release binaries ignore them so an arbitrary process cannot redirect the
  elevated app or switch it to fake readings. The corresponding fix is recorded
  in [`CHANGELOG.md`](../CHANGELOG.md) under 1.1.0.
- The `restart_elevated` command uses OS mechanisms (`ShellExecuteW runas` on
  Windows, `pkexec` on Linux, `osascript` on macOS) and the app manifest
  requests `asInvoker`, so elevation is an explicit user action.
- The Tauri CSP defaults to `default-src 'self'` with narrow exceptions for
  Google Fonts and inline styles; see `tauri.conf.json`.

## Sources

[`vite.config.ts`](../vite.config.ts), [`package.json`](../package.json),
[`.gitignore`](../.gitignore), [`.cargo/config.toml`](../.cargo/config.toml),
[`src/App.tsx`](../src/App.tsx), [`src/components/Dashboard.tsx`](../src/components/Dashboard.tsx),
[`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json),
[`src-tauri/tauri.linux.conf.json`](../src-tauri/tauri.linux.conf.json),
[`src-tauri/tauri.windows.conf.json`](../src-tauri/tauri.windows.conf.json),
[`src-tauri/capabilities/default.json`](../src-tauri/capabilities/default.json),
[`src-tauri/app.manifest`](../src-tauri/app.manifest),
[`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs),
[`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs),
[`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs),
[`src-tauri/worker/src/main.rs`](../src-tauri/worker/src/main.rs),
[`CHANGELOG.md`](../CHANGELOG.md).
