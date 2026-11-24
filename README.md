<h1>
  <img src="public/icon.svg" alt="Zenith" width="58" height="58" style="vertical-align: middle; margin-right: 6px;">
  Zenith
</h1>

Glassmorphic system health monitor for Windows, built with **Tauri v2** and **React 19**. Shows real CPU temperature, per-core temps, CPU/RAM/disk usage, and a top-process list in a small transparent window.

<p>
  <img src="https://img.shields.io/badge/Tauri_v2-FFC131?logo=tauri&logoColor=black" alt="Tauri v2">
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/React_19-61DAFB?logo=react&logoColor=black" alt="React 19">
  <img src="https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Vite-646CFF?logo=vite&logoColor=white" alt="Vite">
  <img src="https://img.shields.io/badge/Tailwind_CSS_4-06B6D4?logo=tailwindcss&logoColor=white" alt="Tailwind CSS">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT">
</p>

> **Platform note:** Developed and tested on Windows 11. Linux/macOS builds may require sensor configuration adjustments.
>
> **Sensor data:** Real package and per-core CPU temperatures are decoded from Intel MSRs by a self-contained native Rust worker (`zenith-sensor-worker`) through the signed **PawnIO** kernel driver — this requires an elevated (administrator) run and the PawnIO driver installed. When the driver is unavailable, Zenith falls back to the labeled **ACPI thermal zone** (shown as `ACPI`, never presented as hardware data). With no temperature source at all, the UI shows `NONE` plus a "Restart as Administrator" prompt. Production builds never fabricate temperatures — the simulated source exists only as a dev-only mock backend (labeled `MOCK`).

## Features

- **Real CPU temperatures** — package and per-core temps decoded from Intel MSRs by a native Rust worker (`zenith-sensor-worker`, ~4.5 MB RSS) through the signed PawnIO kernel driver
- **Source transparency** — the UI always labels where temps come from (`REAL` / `ACPI` / `MOCK` / `NONE`) and prompts an elevated restart when the driver is unreachable
- **Thermal intelligence** — "Throttling Risk", "Repaste Now/Soon", and "Uneven Mount" warnings derived from package temperature and core-delta patterns
- **Dual-ring gauges** — CPU usage and package temperature on one animated, color-coded gauge
- **Per-core grid** — individual core temperatures with status colors
- **System resources** — live RAM/disk usage and the top 5 processes by CPU
- **Native shell** — Rust backend polls hardware via `sysinfo` and streams events to the UI; minimal resource impact

## Visual identity

Zenith keeps its compact transparent dashboard dark and quiet. Cool blue marks
telemetry, cyan marks healthy readings, and orange/magenta/red communicate
thermal warnings and critical states.

| Role | Color |
|---|---|
| Telemetry blue | `#3b83f6` |
| Healthy | `#22d3ee` |
| Warning | `#f97316` |
| Critical | `#ef4444` |

The repository has a product icon but no dashboard screenshot yet. See the
[visual identity guide](docs/visual-identity.md) and status-color source in
[`src/components/Dashboard.tsx`](src/components/Dashboard.tsx).

## Tech Stack

React 19 · TypeScript · Vite · Tailwind CSS v4 · Tauri v2 · Rust (`sysinfo` + native sensor worker + PawnIO driver) · Inter (Google Fonts)

Sensor backends are selected per platform in `src-tauri/worker/src/backend.rs`:
PawnIO/Intel MSR on Windows, hwmon on Linux, a placeholder on macOS, and a
labeled development mock. See [architecture](docs/architecture.md) for the
event pipeline.

## Documentation

| Page | What it covers |
| --- | --- |
| [Docs index](docs/README.md) | Start here; every Zenith guide and canonical source file. |
| [Product](docs/product.md) | Audience, feature inventory, and non-goals. |
| [Architecture](docs/architecture.md) | Stack, process model, and sensor data flow. |
| [Development](docs/development.md) | Setup, conventions, and troubleshooting. |
| [Configuration](docs/configuration.md) | Env vars, worker flags, config files, and security notes. |
| [Testing](docs/testing.md) | Rust test inventory and gaps. |
| [Deployment](docs/deployment.md) | Builds, artifacts, targets, and missing automation. |
| [Changelog](docs/changelog.md) | Pointer to the full release log and notes. |
| [Visual identity](docs/visual-identity.md) | Palette and design source files. |

## Command reference

| Command | Purpose |
| --- | --- |
| `npm run dev` | Vite development server (`vite`). |
| `npm run build` | Type-check and build the frontend (`tsc && vite build`). |
| `npm run preview` | Preview the built frontend (`vite preview`). |
| `npm run tauri` | Tauri v2 CLI; used as `npm run tauri dev` and `npm run tauri build`. |
| `cargo build --release --manifest-path src-tauri/worker/Cargo.toml` | Build the sensor worker; the Tauri dev/build hooks run this automatically. |

There is no `test` script in `package.json`. Automated tests are Rust
`#[test]` functions in the two Cargo crates; see [testing](docs/testing.md).

## Project Structure

```
src/                        React frontend
├── App.tsx                 Title bar, event stream, window controls
├── components/Dashboard.tsx  Gauges, core grid, source badge, metrics, insights
└── App.css                 Global styles (glass effect, animations)

src-tauri/                  Rust backend
├── src/sensor.rs           Sensor polling, payload assembly, temp-source merge (sysinfo + worker)
├── src/worker_supervisor.rs  Spawns/supervises the sensor worker (circuit breaker)
├── src/lib.rs              App setup, sensor thread lifecycle, elevation restart command
├── src/main.rs             Entry point
└── worker/                 zenith-sensor-worker crate — real CPU temps via PawnIO driver
```

## Getting Started

**Prerequisites:** [Node.js](https://nodejs.org/) (LTS), [Rust](https://www.rust-lang.org/tools/install), WebView2 (bundled with Windows 10/11). Real hardware temperatures additionally need the [PawnIO](https://github.com/namazso/PawnIO) driver (`winget install namazso.PawnIO`) and an elevated run.

```bash
git clone https://github.com/Omar-Khaled-57/Zenith.git
cd Zenith
npm install
cargo build --release --manifest-path src-tauri/worker/Cargo.toml   # build the sensor worker (dev only; tauri build does this automatically)
npm run tauri dev   # run as administrator for real (hardware) CPU temperatures
```

## Production Build

```bash
npm run tauri build
```

`tauri build` compiles the frontend, the Rust app, and the sensor worker, and bundles the worker next to `Zenith.exe` via `bundle.resources`. Installers are generated in `src-tauri/target/release/bundle/`.

## Known gaps

- No npm `test` or `lint` script is declared; there are no frontend component
  tests or coverage reports in the repository.
- The icon generator in `scripts/generate-icons.mjs` is not exposed as an npm
  script and expects `icon.svg` in the current working directory.
- No CI/CD, signing, or auto-update configuration for Zenith was found.
- The macOS sensor backend is a placeholder, and Intel-only hardware decode is
  noted in the changelog.

## Author

**Omar Khaled** — [GitHub](https://github.com/Omar-Khaled-57) · [Portfolio](https://omar-el-khouly.vercel.app)

## License

[MIT](LICENSE) © [Omar Khaled](https://github.com/Omar-Khaled-57)
