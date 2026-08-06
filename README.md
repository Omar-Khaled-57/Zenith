<h1>
  <img src="public/icon.svg" alt="Zenith" width="42" height="42" style="vertical-align: middle; margin-right: 6px;">
  Zenith
</h1>

Glassmorphic system health monitor for Windows, built with **Tauri v2** and **React 19**. Shows live CPU temperature, per-core temps, CPU/RAM/disk usage, and a top-process list in a small transparent window.

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
> **Sensor data:** On Windows, package temperature is read from the ACPI thermal zone (`"Computer"` component). Per-core temps and the core-delta readout are shown only when real per-core sensors exist (e.g. hwmon on Linux/macOS); they are hidden rather than fabricated. If no temperature source is available at all, a load-derived simulation is used as a visual fallback.

## Features

- **Thermal intelligence** — "Throttling Risk", "Repaste Now/Soon", and "Uneven Mount" warnings derived from package temperature and core-delta patterns
- **Dual-ring gauges** — CPU usage and package temperature on one animated, color-coded gauge
- **Per-core grid** — individual core temperatures with status colors
- **System resources** — live RAM/disk usage and the top 5 processes by CPU
- **Native shell** — Rust backend polls hardware via `sysinfo` and streams events to the UI; minimal resource impact

## Tech Stack

React 19 · TypeScript · Vite · Tailwind CSS v4 · Tauri v2 · Rust (`sysinfo`) · Inter (Google Fonts)

## Project Structure

```
src/                        React frontend
├── App.tsx                 Title bar, event stream, window controls
├── components/Dashboard.tsx  Gauges, core grid, metrics, insights
└── App.css                 Global styles (glass effect, animations)

src-tauri/                  Rust backend
├── src/sensor.rs           Sensor polling, payload assembly (sysinfo)
├── src/lib.rs              App setup, sensor thread lifecycle
└── src/main.rs             Entry point
```

## Getting Started

**Prerequisites:** [Node.js](https://nodejs.org/) (LTS), [Rust](https://www.rust-lang.org/tools/install), WebView2 (bundled with Windows 10/11).

```bash
git clone https://github.com/Omar-Khaled-57/Zenith.git
cd Zenith
npm install
npm run tauri dev   # run as administrator for full sensor access
```

## Production Build

```bash
npm run tauri build
```

Installers are generated in `src-tauri/target/release/bundle/`.

## Author

**Omar Khaled** — [GitHub](https://github.com/Omar-Khaled-57) · [Portfolio](https://omar-el-khouly.vercel.app)

## License

[MIT](LICENSE) © [Omar Khaled](https://github.com/Omar-Khaled-57)
