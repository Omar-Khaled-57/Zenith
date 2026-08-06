# Zenith Changelog

A running log of every change, fix, and decision during development.

---

## 1.0.0 — Production Release

**Focus**: Performance optimization, accuracy fixes, graceful shutdown, error resilience, and production hardening.

### 🔧 Backend (`src-tauri/`)

| File | What changed |
|------|-------------|
| `src/sensor.rs` | Replaced broken `fast_rand()` (seed ignored, `SystemTime` called every invocation) with `SeededRand` — proper LCG PRNG with deterministic output; throttled process refresh to every 5 polls (reduced from every second); single-pass disk summing (was two `.iter().map()` passes); single-pass core temp min/max (was two `.fold()` calls); RAM/disk units changed to GiB (`÷1_073_741_824`) eliminating double conversion on frontend; removed unnecessary `MINIMUM_CPU_UPDATE_INTERVAL` startup sleep |
| `src/lib.rs` | Added `Arc<AtomicBool>` shutdown signal — sensor thread now exits cleanly on `Exit`/`ExitRequested` events via `RunEvent` handler |

### 🧹 Frontend (`src/`)

| File | What changed |
|------|-------------|
| `src/components/Dashboard.tsx` | **Fixed**: `core_delta` color was inverted (large delta showed cool colors) — added `deltaColor()` function mapping wide deltas to warm colors. **Fixed**: RAM extra `/1024` division removed (back-end now emits GiB directly). **Fixed**: Added `<ErrorBoundary>` with retry on render crash. **Fixed**: `MetricBar` divide-by-zero guard. **Fixed**: `ProcessList` empty-state placeholder and explicit `slice(0,5)` limit. **Fixed**: `formatMem()` now handles sub-MB values (KB fallback). |
| `src/App.tsx` | No changes — event stream and layout were already clean |

### 🐛 Bug Fixes
- **Inverted core_delta color** — `tempColor(pkg_temp - delta)` made large deltas appear cool; replaced with `deltaColor(delta)` showing red for wide spreads.
- **RAM display off by ~2.4%** — backend sent MiB, frontend did `÷1024` again, now sent directly in GiB.
- **Sensor thread never exits** — `loop` with no stop condition; now terminates cleanly on app close.
- **`fast_rand()` seed was a no-op** — every call sampled `SystemTime::now()` making the parameter meaningless; replaced with stateful `SeededRand::next()`.
- **Process list refreshed every second** — `refresh_processes(All, true)` every poll; now throttled to every 5 seconds.
- **No error boundary in React** — render crash would blank the app silently; now caught with retry button.

### ⚡ Performance
- **~80% reduction in process enumeration** — from every poll to every 5th poll.
- **~200ms faster startup** — removed initial `MINIMUM_CPU_UPDATE_INTERVAL` sleep.
- **Fewer allocations per poll** — single-pass disk/minmax, `sort_unstable_by` instead of stable sort.

### 🏭 Production Readiness
- All icon assets generated from custom SVG at every required size (52 files: bundle PNGs, ICO, ICNS, Windows Store tiles, iOS, Android).
- `tauri.conf.json` references all icons explicitly; NSIS installer/uninstaller icons and WiX icon configured.
- Custom app icon (`icon.svg`) used as favicon in `index.html`.
- No default Tauri icon assets remain in use.

### 🔬 Accuracy & Hardening Audit

Second pass before shipping — sensor accuracy, honest data presentation, dependency hygiene, security, and code quality.

### 🔧 Backend (`src-tauri/`)

| File | What changed |
|------|-------------|
| `src/sensor.rs` | **Fixed**: On Windows, the real ACPI thermal-zone temperature (sysinfo label `"Computer"`) is now used as the package temperature — previously the label matcher never matched it, so the real reading was discarded and temperatures were always simulated. Simulation now only runs when **no** real temperature source exists. **Fixed**: first-sample accuracy — CPU and process usage are warmed up at startup (sysinfo requires two refreshes ≥ `MINIMUM_CPU_UPDATE_INTERVAL` apart), and the throttled process refresh now lands on poll #2 so the very first emitted payload is valid instead of all-zero. **Fixed**: per-process CPU normalized from per-core % (could read e.g. 800%) to a 0–100% share of total CPU, matching the global gauge and Task Manager. **Fixed**: `memory_mb` renamed to `memory_gb` (the value was always GiB). **Fixed**: per-core matching now excludes non-CPU sensors (e.g. "GPU Core"). Clippy warnings cleared (`.clamp`, redundant `as f32` casts). |
| `src/lib.rs` | Removed unused `tauri-plugin-store` registration |
| `Cargo.toml` | Removed unused `tauri-plugin-store` and `serde_json` direct dependencies |

### 🧹 Frontend (`src/`)

| File | What changed |
|------|-------------|
| `src/components/Dashboard.tsx` | **Fixed**: ring gauges now animate via `stroke-dashoffset` (the CSS already transitioned it; the old `strokeDasharray`-length approach was not animatable). **Fixed**: Δ readout hidden when no real per-core data exists (temps < 2), so it cannot show a fabricated `0.0°`/simulated delta |
| `src/App.tsx` | Renamed `memory_mb` → `memory_gb` to match the backend payload |

### 🔒 Security
- **CSP added** — `tauri.conf.json` `csp: null` replaced with a restrictive policy (`default-src 'self'` + Google Fonts + inline styles only). Tauri IPC runs via host-injected scripts and is unaffected.
- Removed committed build/debug junk (`build_msg.txt`, `errors.txt`, `last_error.tx`, `log.txt`) and gitignored them.

### 🏭 Dependencies
- `npm audit` vulnerabilities reduced from 4 (2 high) to 0.
- Cargo dependency tree trimmed (store plugin removed).

### 📄 Docs
- `README.md` — sensor-source behavior now documented accurately (ACPI thermal zone on Windows; per-core data only where real sensors exist); admin-mode wording softened.
- `README.md` — release artifacts now published under `dev/release/{version}`; installers kept in `dev/release/1.0.0/`.

### 📦 Packaging
- First Windows release published to `dev/release/1.0.0/`:
  - `Zenith_1.0.0_x64_en-US.msi` — MSI (WiX) installer
  - `Zenith_1.0.0_x64-setup.exe` — NSIS installer
  - `RELEASE_NOTES.md` — release note with title
- Installer source-of-truth remains `src-tauri/target/release/bundle/`.

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
