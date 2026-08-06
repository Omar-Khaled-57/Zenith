import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Dashboard from "./components/Dashboard";
import "./App.css";

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_gb: number;
}

export interface SensorPayload {
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

const WINDOW_BTN_STYLE: React.CSSProperties = {
  width: 14,
  height: 14,
  borderRadius: "50%",
  border: "none",
  cursor: "pointer",
  opacity: 0.7,
  transition: "opacity 0.2s",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  color: "#fff",
  fontSize: 10,
  lineHeight: 1,
  fontWeight: "bold",
};

const STYLES = {
  titleBar: {
    paddingLeft: 8,
    paddingRight: 8,
    userSelect: "none",
    WebkitUserSelect: "none",
  } as React.CSSProperties,
  content: {
    paddingTop: 38,
    paddingLeft: 18,
    paddingRight: 18,
    marginBottom: 14,
  } as React.CSSProperties,
  zenithLabel: {
    fontSize: 11,
    fontWeight: 700,
    letterSpacing: "0.2em",
    color: "rgba(148,163,184,0.8)",
    textTransform: "uppercase",
  } as React.CSSProperties,
} as const;

function WindowControls() {
  const handleMinimize = useCallback(async () => {
    try {
      await getCurrentWindow().minimize();
    } catch { /* ignored */ }
  }, []);

  const handleClose = useCallback(async () => {
    try {
      await getCurrentWindow().close();
    } catch { /* ignored */ }
  }, []);

  return (
    <div className="flex items-center gap-2" onMouseDown={(e) => e.stopPropagation()}>
      <button
        id="btn-minimize"
        onClick={handleMinimize}
        style={{ ...WINDOW_BTN_STYLE, background: "#d946ef" }}
        onMouseEnter={(e) => (e.currentTarget.style.opacity = "1")}
        onMouseLeave={(e) => (e.currentTarget.style.opacity = "0.7")}
        aria-label="Minimize"
      >
        -
      </button>
      <button
        id="btn-close"
        onClick={handleClose}
        style={{ ...WINDOW_BTN_STYLE, background: "#ef4444" }}
        onMouseEnter={(e) => (e.currentTarget.style.opacity = "1")}
        onMouseLeave={(e) => (e.currentTarget.style.opacity = "0.7")}
        aria-label="Close"
      >
        ×
      </button>
    </div>
  );
}

function Logo() {
  return (
    <div className="flex items-center gap-2" data-tauri-drag-region="true">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" data-tauri-drag-region="true">
        <circle cx="12" cy="12" r="9" stroke="#22d3ee" strokeWidth="1.5" opacity="0.5" />
        <circle cx="12" cy="12" r="5" stroke="#d946ef" strokeWidth="1.5" />
        <circle cx="12" cy="12" r="2" fill="#22d3ee" />
      </svg>
      <span style={STYLES.zenithLabel} data-tauri-drag-region="true">
        ZENITH
      </span>
    </div>
  );
}

function TitleBar() {
  return (
    <div
      className="drag-region flex items-center justify-between pt-6 pb-4 cursor-move"
      data-tauri-drag-region="true"
      style={STYLES.titleBar}
    >
      <Logo />
      <WindowControls />
    </div>
  );
}

function App() {
  const [sensorData, setSensorData] = useState<SensorPayload | null>(null);

  useEffect(() => {
    const unlisten = listen<SensorPayload>("system-metrics", (event) => {
      setSensorData(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="w-screen h-screen flex flex-col relative" style={{ fontFamily: "'Inter', sans-serif" }}>
      <div className="glass flex flex-col flex-1 overflow-hidden relative">
        <TitleBar />
        <div className="flex-1 flex flex-col overflow-y-auto" style={STYLES.content}>
          <Dashboard data={sensorData} />
        </div>
      </div>
    </div>
  );
}

export default App;
