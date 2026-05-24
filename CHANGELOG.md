# Zenith Changelog

A running log of every change, fix, and decision during development.

---

## 0.1.1 — Code Quality & Stability Overhaul

**Focus**: Optimize performance, eliminate technical debt, enforce consistent patterns across frontend and backend, and refine the thermal intelligence pipeline.

### 🚀 New Features
- **`CHANGELOG.md`** — this public dev log at the project root

### 🧹 Backend (`src-tauri/`)

| File | What changed |
|------|-------------|
| `src/lib.rs` | Moved `use tauri::Emitter` to top-level import; named sensor thread `"sensor-poll"`; added `eprintln` error handling on `emit()` failure |
| `src/sensor.rs` | Removed redundant `cpu_temp` field from `SensorPayload`; replaced fragile `subsec_nanos()` PRNG with proper LCG-based `fast_rand()`; extracted `cpu_load_to_temp()` for clean load→temp mapping; replaced `f32::MIN`/`f32::MAX` with `f32::NEG_INFINITY`/`f32::INFINITY`; used named byte constants (`1_048_576.0`, `1_073_741_824.0`); cleaned up component temperature reading pattern (`let Some` instead of `match`) |

### 🧹 Frontend (`src/`)

| File | What changed |
|------|-------------|
| `src/App.tsx` | Removed all `as any` casts — typed all inline styles as `React.CSSProperties`; extracted `WindowControls`, `Logo`, `TitleBar` as separate components; used `useCallback` for window handlers; added `aria-label` on minimize/close buttons |
| `src/components/Dashboard.tsx` | Extracted `LoadingState`, `InsightBadge`, `GaugeSection`, `Divider` components; created shared style token object `S` to deduplicate inline style objects; added `formatMem()` helper; replaced many inline styles with Tailwind utility classes (grid, flex, text-center); core grid uses `grid-cols-4` instead of inline `repeat(min(temps.length,4))`; removed `cpu_temp` references to match updated `SensorPayload` |
| `src/App.css` | Unchanged (animations, glass style, Tailwind import untouched) |

### 🧹 Config & Cleanup

| File | What changed |
|------|-------------|
| `tailwind.config.js` | **Removed** — Tailwind v4 uses CSS-first config via `@import "tailwindcss"` |
| `postcss.config.js` | **Removed** — Tailwind v4 is loaded via `@tailwindcss/vite` plugin, not PostCSS |
| `src/assets/react.svg` | **Removed** — unused asset |
| `public/vite.svg` | **Removed** — unused favicon |
| `public/tauri.svg` | **Removed** — unused asset |
| `index.html` | Updated favicon to reference `icon.svg` instead of removed `vite.svg` |
| `.gitignore` | Added `output/`; cleaned up duplicate Rust/Cargo entries; removed incorrect `cargo-lock.json` entry |

### 🐛 Bug Fixes
- **Rust type mismatch** — `as_nanos()` returns `u128`, PRNG XOR with `u64` seed failed. Fixed with `u128::from(seed)`.
- **Rust `temperature()` return type** — method returns `Option<f32>`, not `Result`. Fixed `let Ok(temp)` → `let Some(temp)`.
- **Simulation realism** — removed unrealistic deliberate delta forcing on core 4. Simulation now produces natural 0–14°C core-to-core variation via independent jitter + spread per core.

### 🎯 Thermal Intelligence
- **Simulation engine** — rewrote with LCG-based `fast_rand(seed)` for better distribution across rapid calls; core temps vary naturally by `(jitter * 10) + (spread * 4)` degrees
- **Insight thresholds** — preserved existing logic (THROTTLING RISK at >90°C, REPASTE NOW at >80% CPU + >15°C delta, etc.), now driven by more realistic simulated data on machines without hardware sensors

### 💡 Considered But Not Done
- **Extract Rust `SensorEngine` trait** — single implementation is sufficient for now
- **Add Tauri commands for manual sensor refresh** — event-stream model is simpler and sufficient
- **CSS variables for temperature colors** — `tempColor()` function works reliably and keeps color logic in one place
- **Virtual scrolling for process list** — only 5 items displayed, not worth the complexity
