# 🌌 Zenith — Thermal Monitor

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0-magenta.svg)
![Tauri](https://img.shields.io/badge/Tauri-v2-cyan.svg)
![React](https://img.shields.io/badge/React-19-blue.svg)
![TypeScript](https://img.shields.io/badge/TypeScript-5.8-blue.svg)

**Zenith** is a high-performance, glassmorphic system health monitor built with **Tauri v2** and **React**. Designed for enthusiasts who demand real-time thermal intelligence, Zenith provides deep insights into your system's hardware state with a stunning, futuristic interface.

---

## ✨ Key Features

- **🛡️ Thermal Intelligence**: Real-time analysis of CPU health, offering "Repaste Soon" or "Uneven Mount" warnings based on core delta and thermal patterns.
- **🌀 Precision Gauges**: Dual-ring gauges for simultaneous monitoring of CPU Usage and Package Temperature.
- **💎 Glassmorphic UI**: A premium, transparent interface featuring blur effects, Cyan/Magenta accents, and smooth animations.
- **🌡️ Multi-Core Monitoring**: Individual core temperature tracking in a clean, high-density grid.
- **🚀 Resource Tracking**: Live metrics for RAM, Disk usage, and a "Top Demand" process list.
- **📦 Native Shell**: Lightweight Rust-powered backend ensuring minimal impact on system resources.

---

## 🛠️ Tech Stack

- **Frontend**: [React 19](https://react.dev/), [Vite](https://vitejs.dev/), [Tailwind CSS v4](https://tailwindcss.com/)
- **Backend/Shell**: [Tauri v2](https://tauri.app/), [Rust](https://www.rust-lang.org/)
- **Icons**: Custom SVG & [Lucide React](https://lucide.dev/)
- **Typography**: [Inter](https://rsms.me/inter/)

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (Latest LTS)
- [Rust](https://www.rust-lang.org/tools/install)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (Included in Windows 10/11)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/Omar-Khaled-57/Zenith.git
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Run the development server:
   ```bash
   npm run tauri dev
   ```

### Building for Production

To create a optimized, installer package:
```bash
npm run tauri build
```
The installers will be located in `src-tauri/target/release/bundle/`.

---

## 👨‍💻 Author

**Omar Khaled**
GitHub: [@Omar-Khaled-57](https://github.com/Omar-Khaled-57)

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

