# Zenith architecture

## Stack

| Layer | Technology | Evidence |
| --- | --- | --- |
| UI | React 19, TypeScript, Vite 7, Tailwind CSS v4 via `@tailwindcss/vite` | [`package.json`](../package.json), [`vite.config.ts`](../vite.config.ts), [`src/App.css`](../src/App.css) |
| Font | Inter loaded from Google Fonts | [`index.html`](../index.html) |
| Desktop shell | Tauri v2 (`@tauri-apps/api`, `@tauri-apps/cli`, `tauri-plugin-opener`) | [`package.json`](../package.json), [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml) |
| Rust app | `zenith_lib`, `sysinfo` 0.38.4, serde/serde_json, platform `windows-sys` | [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml) |
| Sensor worker | `zenith-sensor-worker` crate with JSONL protocol, Intel MSR decode, PawnIO access on Windows, hwmon on Linux, a mock backend, and a macOS stub | [`src-tauri/worker/Cargo.toml`](../src-tauri/worker/Cargo.toml), [`src-tauri/worker/src/lib.rs`](../src-tauri/worker/src/lib.rs) |

## Entry points

| Entry | Role |
| --- | --- |
| [`src/main.tsx`](../src/main.tsx) | Mounts `<App />` into `#root` from [`index.html`](../index.html). |
| [`src/App.tsx`](../src/App.tsx) | Listens for `system-metrics`, owns the latest payload in React state, renders the title bar and dashboard. |
| [`src-tauri/src/main.rs`](../src-tauri/src/main.rs) | Native entry point; calls `zenith_lib::run()` and suppresses the Windows console. |
| [`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs) | Tauri builder, `restart_elevated` command, setup of the sensor thread, shutdown handling. |
| [`src-tauri/worker/src/main.rs`](../src-tauri/worker/src/main.rs) | Worker binary loop: opens a backend, samples once per second, writes JSONL to stdout. |

## Directory map

| Path | Responsibility |
| --- | --- |
| `src/` | Frontend: `App.tsx`, `App.css`, `main.tsx`, `components/Dashboard.tsx`, `vite-env.d.ts`. |
| `src-tauri/src/` | App lifecycle (`main.rs`, `lib.rs`), sensor aggregation (`sensor.rs`), worker supervision (`worker_supervisor.rs`). |
| `src-tauri/worker/src/` | `backend.rs` backend trait and platform selection, `hardware_access.rs`, `intel.rs` decode, `pawnio.rs`, `linux_hwmon.rs`, `macos_stub.rs`, `mock.rs`, `protocol.rs`, `supervisor.rs`. |
| `src-tauri/worker/tests/` | CLI integration tests for the worker binary. |
| `src-tauri/capabilities/` | Tauri permission set for the main window. |
| `src-tauri/icons/`, `public/icon.svg` | Bundled platform icons and the source SVG used by the UI favicon. |
| `src-tauri/gen/schemas/` | Generated Tauri capability schemas; ignored by [`src-tauri/.gitignore`](../src-tauri/.gitignore). |
| `scripts/generate-icons.mjs` | Standalone icon generator using `sharp`; not exposed as an npm script. |
| `dev/` | Ignored development references: LibreHardwareMonitor and PawnIO source copies, plans, probe, and release notes. See [`.gitignore`](../.gitignore). |
| `docs/` | This documentation set, including the pre-existing visual identity guide. |

## Process and thread model

1. `zenith_lib::run()` builds the Tauri app with the opener plugin, the
   `restart_elevated` command, and a window configured as 320×550, undecorated,
   transparent, and non-resizable in
   [`tauri.conf.json`](../src-tauri/tauri.conf.json).
2. During setup, a named `sensor-poll` thread starts a `WorkerSupervisor`, then
   polls once per second and emits `system-metrics` to the webview.
3. `WorkerSupervisor::start` spawns a `worker-supervisor` controller thread. It
   launches `zenith-sensor-worker` as a child process with piped stdin/stdout,
   a reader thread for JSONL lines, and a 3-second hang timeout; a circuit
   breaker uses 250 ms, 1 s, and 5 s backoff before giving up.
4. `SensorEngine::poll` merges the latest worker message with the labeled ACPI
   fallback and assembles the payload. On Tauri `Exit` or `ExitRequested`, the
   running flag is cleared, the supervisor stops, and the worker is killed.
   The worker also watches stdin for EOF as its shutdown signal.

## Data and state flow

```text
OS / sensor backend
  -> zenith-sensor-worker (JSONL envelope on stdout)
  -> worker-supervisor (latest message + last error)
  -> SensorEngine::poll (source merge, CPU/RAM/disk/processes)
  -> Tauri event "system-metrics"
  -> React useState in App.tsx
  -> Dashboard.tsx gauges, grid, bars, process list, insight badge
```

Only the newest payload is kept; there is no history buffer. The frontend
payload shape is declared in `App.tsx` and mirrored by `SensorPayload` in
`sensor.rs`. The worker protocol is version 1 in
[`protocol.rs`](../src-tauri/worker/src/protocol.rs): sample and error envelopes
carry `Source` (`hardware`, `mock`, `none`), `Status` (`healthy`, `degraded`,
`unavailable`), per-sensor `Quality`, CPU info, derived hottest/coolest/delta,
and error codes such as `DRIVER_UNAVAILABLE`.

Temperature source selection in `merge_temps`:

- A healthy worker sample wins, keeping the last finite package value and
  dropping non-finite core values; its source is reported as `hardware`,
  `mock`, or `none`.
- Otherwise, ACPI components with labels such as `computer`, `package`, or
  `tctl` supply a package temperature, and `core` labels supply per-core values;
  this path is reported as `acpi`/`degraded`.
- With neither source available, the payload reports `none`/`unavailable` and
  any worker error string.

CPU/RAM/disk and process values come from `sysinfo`. Process CPU percentages are
normalized by the logical CPU count and capped at 100, sorted by usage, and
limited to five entries. Process refreshes are throttled to every fifth poll;
memory and disk totals are converted to GiB in the backend.

## Platform backends

| Platform | Backend | Notes |
| --- | --- | --- |
| Windows | `WinMsrBackend` via PawnIO | Opens `\\.\GLOBALROOT\Device\PawnIO`, loads `res/IntelMSR.bin`, decodes Intel MSRs. Requires the PawnIO driver and an elevated run for real temperatures. |
| Linux | `LinuxHwmonBackend` | Reads `/sys/class/hwmon` `coretemp`/`k10temp` inputs; no separate privileged driver in the source comments. |
| macOS | `MacosBackend` | Placeholder that reports `DriverUnavailable`; real SMC/IOKit work is noted as a follow-up. |
| Any, dev only | `MockBackend` | Selected by `--mock`; always labeled `MOCK` in the UI. |

## Storage and integration boundaries

- No database, HTTP API, local storage keys, or user-settings persistence were
  found in the application source. The only webview state is the current event
  payload in `App.tsx`.
- OS integrations: Tauri window APIs, `sysinfo`, Windows ShellExecute/`pkexec`/
  `osascript` for elevation, the PawnIO device on Windows, and hwmon sysfs on
  Linux.
- External network request: the Google Fonts stylesheet declared in
  `index.html`; the Tauri CSP in `tauri.conf.json` permits Google Fonts and
  inline styles while defaulting other sources to `self`.

## Sources

[`package.json`](../package.json), [`index.html`](../index.html),
[`vite.config.ts`](../vite.config.ts), [`src/App.tsx`](../src/App.tsx),
[`src/components/Dashboard.tsx`](../src/components/Dashboard.tsx),
[`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json),
[`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml),
[`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs),
[`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs),
[`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs),
[`src-tauri/worker/src/backend.rs`](../src-tauri/worker/src/backend.rs),
[`src-tauri/worker/src/protocol.rs`](../src-tauri/worker/src/protocol.rs),
[`src-tauri/worker/src/main.rs`](../src-tauri/worker/src/main.rs),
[`src-tauri/worker/src/supervisor.rs`](../src-tauri/worker/src/supervisor.rs).
