import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Dashboard from "./components/Dashboard";
import "./App.css";

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_mb: number;
}

export interface SensorPayload {
  cpu_temp: number;
  cpu_pkg_temp: number;
  cpu_usage: number;
  core_delta: number;
  temps: number[];
  ram_usage: number;
  ram_total: number;
  disk_usage: number;
  disk_total: number;
  top_processes: ProcessInfo[];
}

function App() {
  const [sensorData, setSensorData] = useState<SensorPayload | null>(null);

  useEffect(() => {
    const unlisten = listen<SensorPayload>("system-metrics", (event) => {
      setSensorData(event.payload);
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  const handleMinimize = async () => {
    try {
      const win = getCurrentWindow();
      await win.minimize();
    } catch (_) { }
  };

  const handleClose = async () => {
    try {
      const win = getCurrentWindow();
      await win.close();
    } catch (_) { }
  };

  return (
    <div className="w-screen h-screen flex flex-col relative" style={{ fontFamily: "'Inter', sans-serif" }}>
      {/* Glassmorphic main container */}
      <div className="glass flex flex-col flex-1 overflow-hidden relative">

        {/* Title bar / drag region */}
        <div
          className="flex items-center justify-between pt-6 pb-4 cursor-move"
          data-tauri-drag-region="true"
          style={{ WebkitAppRegion: "drag", userSelect: "none", paddingLeft: "8px", paddingRight: "8px" } as any}
        >
          <div className="flex items-center gap-2" data-tauri-drag-region="true">
            {/* Logo mark */}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" data-tauri-drag-region="true">
              <circle cx="12" cy="12" r="9" stroke="#22d3ee" strokeWidth="1.5" opacity="0.5" />
              <circle cx="12" cy="12" r="5" stroke="#d946ef" strokeWidth="1.5" />
              <circle cx="12" cy="12" r="2" fill="#22d3ee" />
            </svg>
            <span style={{
              fontSize: 11,
              fontWeight: 700,
              letterSpacing: "0.2em",
              color: "rgba(148,163,184,0.8)",
              textTransform: "uppercase"
            }} data-tauri-drag-region="true">
              ZENITH
            </span>
          </div>

          {/* Window controls */}
          <div className="flex items-center gap-2" onMouseDown={(e) => e.stopPropagation()}>
            <button
              id="btn-minimize"
              onClick={handleMinimize}
              style={{
                width: 14, height: 14,
                borderRadius: "50%",
                background: "#d946ef",
                border: "none",
                cursor: "pointer",
                opacity: 0.7,
                transition: "opacity 0.2s",
                WebkitAppRegion: "no-drag",
                display: "flex", alignItems: "center", justifyContent: "center",
                color: "#fff", fontSize: 10, lineHeight: 1, fontWeight: "bold"
              } as any}
              onMouseEnter={e => (e.currentTarget.style.opacity = "1")}
              onMouseLeave={e => (e.currentTarget.style.opacity = "0.7")}
            >
              -
            </button>
            <button
              id="btn-close"
              onClick={handleClose}
              style={{
                width: 14, height: 14,
                borderRadius: "50%",
                background: "#ef4444",
                border: "none",
                cursor: "pointer",
                opacity: 0.7,
                transition: "opacity 0.2s",
                WebkitAppRegion: "no-drag",
                display: "flex", alignItems: "center", justifyContent: "center",
                color: "#fff", fontSize: 10, lineHeight: 1, fontWeight: "bold"
              } as any}
              onMouseEnter={e => (e.currentTarget.style.opacity = "1")}
              onMouseLeave={e => (e.currentTarget.style.opacity = "0.7")}
            >
              ×
            </button>
          </div>
        </div>

        {/* Main content */}
        <div className="flex-1 flex flex-col overflow-y-auto" style={{ paddingTop: "38px", paddingLeft: "18px", paddingRight: "18px", marginBottom: "14px" }}>
          <Dashboard data={sensorData} />
        </div>
      </div>
    </div>
  );
}

export default App;
