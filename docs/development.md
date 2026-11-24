# Zenith development

## Prerequisites

| Requirement | Evidence |
| --- | --- |
| Node.js (LTS) and npm | [`README.md`](../README.md) getting-started section; npm scripts in [`package.json`](../package.json). |
| Rust toolchain | [`README.md`](../README.md); the worker crate declares `rust-version = "1.75"` in [`worker/Cargo.toml`](../src-tauri/worker/Cargo.toml). |
| Windows 10/11 with WebView2 | [`README.md`](../README.md) states WebView2 is bundled with Windows 10/11. |
| PawnIO driver for real CPU temperatures on Windows | [`README.md`](../README.md) documents `winget install namazso.PawnIO`; an elevated run is required. |

## Install and run

From the project root:

```bash
npm install
cargo build --release --manifest-path src-tauri/worker/Cargo.toml
npm run tauri dev
```

These commands are taken from the project README; the worker build is marked
there as a development-only step because `tauri build` performs it
automatically. Run elevated when hardware temperatures are needed.
[`docs/architecture.md`](./architecture.md) explains the other platform
backends.

## Command reference

| Command | Source | Effect |
| --- | --- | --- |
| `npm run dev` | [`package.json`](../package.json) | `vite` development server. |
| `npm run build` | [`package.json`](../package.json) | `tsc && vite build` (also the Tauri `beforeBuildCommand` prefix). |
| `npm run preview` | [`package.json`](../package.json) | `vite preview` of the built frontend. |
| `npm run tauri` | [`package.json`](../package.json) | Tauri v2 CLI entry point; the README uses `npm run tauri dev` and `npm run tauri build`. |
| `cargo build --release --manifest-path src-tauri/worker/Cargo.toml` | [`README.md`](../README.md), [`tauri.conf.json`](../src-tauri/tauri.conf.json) | Builds the standalone sensor worker; the Tauri dev/build hooks invoke the same command. |
| `npm run tauri build` | [`README.md`](../README.md) | Frontend build, Rust build, worker build, and bundling. |

There is **no `test`, `typecheck`, or `lint` script** in `package.json`. Rust
tests are described in [testing](./testing.md).

## Repository structure

| Path | Contents |
| --- | --- |
| `src/` | React UI and styles. |
| `src-tauri/` | Tauri configuration, Rust app crate, capabilities, icons, generated schemas, and the `worker/` crate. |
| `scripts/generate-icons.mjs` | Standalone `sharp`-based icon generator; see the note below. |
| `public/icon.svg` | Product icon and favicon source. |
| `dev/` | Ignored reference material: LibreHardwareMonitor, PawnIO, PawnIO modules, plans, probe, and release notes. |
| `docs/` | This documentation set. |
| `dist/`, `src-tauri/target/`, `src-tauri/worker/target/`, `src-tauri/gen/` | Generated build output, all ignored by [`.gitignore`](../.gitignore). |

## Conventions

- TypeScript is strict with `noEmit`, bundler resolution, and React JSX; see
  [`tsconfig.json`](../tsconfig.json). Vite uses the React and Tailwind v4
  plugins in [`vite.config.ts`](../vite.config.ts).
- The dashboard uses Tailwind utility classes for layout plus inline style
  objects for telemetry colors and ring geometry. Thermal thresholds and
  colors live in `getInsight`, `tempColor`, and `deltaColor` in
  [`Dashboard.tsx`](../src/components/Dashboard.tsx).
- The frontend `SensorPayload` interface in [`App.tsx`](../src/App.tsx) mirrors
  the Rust `SensorPayload` in [`sensor.rs`](../src-tauri/src/sensor.rs). Keep
  both in sync when changing the event contract.
- Worker messages are versioned JSONL defined in
  [`protocol.rs`](../src-tauri/worker/src/protocol.rs); new sensors should be
  added there and exposed through the `SensorBackend` trait in
  [`backend.rs`](../src-tauri/worker/src/backend.rs).
- Temperature sources are explicit: worker samples are `hardware` (`REAL` in
  the UI), ACPI is labeled `ACPI`, the dev mock is `MOCK`, and missing data is
  `NONE`. Do not present simulated or zone temperatures as hardware readings.

## Adding a feature

1. If the change affects telemetry, update `SensorPayload` in both
   `App.tsx` and `src-tauri/src/sensor.rs`, and the worker protocol if a new
   sensor field is involved.
2. Implement UI changes in `src/components/Dashboard.tsx`, reusing the
   existing error boundary and loading state.
3. Add or update Rust tests beside the changed logic and worker CLI tests for
   flag changes; see [testing](./testing.md).
4. Update the relevant pages in `docs/` and `CHANGELOG.md` if the behavior is
   user-visible.

## Troubleshooting

| Symptom | Evidence-based check |
| --- | --- |
| Temperature badge shows `NONE` | No worker sample and no ACPI package reading were available. The README and `ElevationBanner` call for the PawnIO driver and an elevated restart. |
| Temperature badge shows `ACPI` | The worker did not supply a healthy sample; `merge_temps` selected the labeled zone fallback. |
| Worker cannot be found | `resolve_worker_exe` checks `ZENITH_WORKER_PATH` in debug builds, the bundled resource directory, the executable directory, then `src-tauri/worker/target/{release,debug}` before falling back to the bare binary name. |
| Worker stops responding | The supervisor applies a 3-second hang timeout and the circuit-breaker backoff in [`worker/src/supervisor.rs`](../src-tauri/worker/src/supervisor.rs). |
| Restart-as-admin under `tauri dev` leaves the old window open | By design: debug builds hide the window so the Vite server survives; release builds exit. See the comment in [`lib.rs`](../src-tauri/src/lib.rs). |
| Icon regeneration fails from the project root | `scripts/generate-icons.mjs` reads `icon.svg` relative to the current working directory and writes to `src-tauri/icons`; the committed SVG is at `public/icon.svg`, and the script has no npm script wrapper. No verified invocation is documented. |

## Sources

[`README.md`](../README.md), [`package.json`](../package.json),
[`.gitignore`](../.gitignore), [`tsconfig.json`](../tsconfig.json),
[`vite.config.ts`](../vite.config.ts), [`scripts/generate-icons.mjs`](../scripts/generate-icons.mjs),
[`src/App.tsx`](../src/App.tsx), [`src/components/Dashboard.tsx`](../src/components/Dashboard.tsx),
[`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs),
[`src-tauri/src/sensor.rs`](../src-tauri/src/sensor.rs),
[`src-tauri/src/worker_supervisor.rs`](../src-tauri/src/worker_supervisor.rs).
