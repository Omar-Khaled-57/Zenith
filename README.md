<h1>
  <img src="icon.svg" alt="Zenith" width="42" height="42" style="vertical-align: middle; margin-right: 6px;">
  Zenith
</h1>

High-performance, glassmorphic system health monitor built with **Tauri v2** and **React**. Real-time thermal intelligence with a stunning, futuristic interface.

<p>
  <img src="https://img.shields.io/badge/Tauri_v2-FFC131?logo=tauri&logoColor=black" alt="Tauri v2">
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/React_19-61DAFB?logo=react&logoColor=black" alt="React 19">
  <img src="https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Vite-646CFF?logo=vite&logoColor=white" alt="Vite">
  <img src="https://img.shields.io/badge/Tailwind_CSS_4-06B6D4?logo=tailwindcss&logoColor=white" alt="Tailwind CSS">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT">
</p>

> **Platform note:** Developed and tested on Windows 11. Linux and macOS builds may require sensor configuration adjustments.

---

## Features

| Feature | Description |
|---|---|
| **Thermal Intelligence** | Real-time CPU health analysis with "Repaste Soon" and "Uneven Mount" warnings based on core delta patterns |
| **Precision Gauges** | Dual-ring gauges for simultaneous CPU Usage and Package Temperature monitoring |
| **Glassmorphic UI** | Premium transparent interface with blur effects, Cyan/Magenta accents, and smooth animations |
| **Multi-Core Monitoring** | Individual core temperature tracking in a clean, high-density grid |
| **Resource Tracking** | Live metrics for RAM, Disk usage, and a "Top Demand" process list |
| **Native Shell** | Lightweight Rust-powered backend with minimal resource impact |

## Tech Stack

| Layer | Library |
|---|---|
| **Frontend** | [React 19](https://react.dev/), [Vite](https://vitejs.dev/), [Tailwind CSS v4](https://tailwindcss.com/) |
| **Desktop Shell** | [Tauri v2](https://tauri.app/), [Rust](https://www.rust-lang.org/) |
| **Icons** | Custom SVG + [Lucide React](https://lucide.dev/) |
| **Typography** | [Inter](https://rsms.me/inter/) |

## Structure

```
Zenith/
├── src/
│   ├── components/
│   │   └── Dashboard.tsx    # Main dashboard with gauges, core grid, metrics
│   ├── App.tsx              # Root component
│   ├── App.css              # Global styles
│   ├── main.tsx             # Entry point
│   └── vite-env.d.ts
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Tauri entry
│   │   ├── lib.rs           # Command handlers
│   │   └── sensor.rs        # Hardware sensor polling
│   ├── icons/               # Platform icons
│   ├── Cargo.toml
│   └── tauri.conf.json      # Tauri configuration
├── dev/release/             # Build artifacts
├── scripts/
│   └── generate-icons.mjs
├── icon.svg                 # App icon
├── index.html
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (Latest LTS)
- [Rust](https://www.rust-lang.org/tools/install)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (Included in Windows 10/11)

### Install

```bash
git clone https://github.com/Omar-Khaled-57/Zenith.git
cd Zenith
npm install
```

### Development

Run in **administrator mode** (required for hardware sensor access):

```bash
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

Installers will be at `src-tauri/target/release/bundle/`.

## Author

**Omar Khaled** — [GitHub](https://github.com/Omar-Khaled-57) · [Portfolio](https://omar-el-khouly.vercel.app)

## License

[MIT](LICENSE) © [Omar Khaled](https://github.com/Omar-Khaled-57)
