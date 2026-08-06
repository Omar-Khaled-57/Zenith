import { Component, ErrorInfo, ReactNode } from "react";
import { SensorPayload } from "../App";

// ── Error Boundary ──

interface ErrorBoundaryProps { children: ReactNode; }
interface ErrorBoundaryState { hasError: boolean; error: Error | null; }

class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }
  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }
  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Dashboard error:", error, info);
  }
  render() {
    if (this.state.hasError) {
      return (
        <div className="flex-1 flex flex-col items-center justify-center gap-3 px-4">
          <svg width="36" height="36" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#ef4444" strokeWidth="1.5" opacity="0.4" />
            <path d="M12 7v4M12 14v.01" stroke="#ef4444" strokeWidth="2" strokeLinecap="round" />
          </svg>
          <span style={{ fontSize: 10, letterSpacing: "0.1em", color: "rgba(239,68,68,0.7)" }}>
            DASHBOARD ERROR
          </span>
          <button
            onClick={() => this.setState({ hasError: false, error: null })}
            style={{
              padding: "4px 14px", borderRadius: 6, border: "1px solid rgba(255,255,255,0.1)",
              background: "rgba(255,255,255,0.05)", color: "rgba(203,213,225,0.7)",
              fontSize: 10, cursor: "pointer",
            }}
          >
            RETRY
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

// ── Thermal Intelligence ──

interface Insight {
  label: string;
  sublabel: string;
  color: string;
  glowColor: string;
  severity: "ok" | "warn" | "danger" | "critical";
}

function getInsight(data: SensorPayload): Insight {
  const { cpu_pkg_temp, cpu_usage, core_delta } = data;

  if (cpu_pkg_temp > 90) {
    return {
      label: "THROTTLING RISK",
      sublabel: "Critical temperature — check airflow immediately",
      color: "#ef4444",
      glowColor: "rgba(239,68,68,0.25)",
      severity: "critical",
    };
  }
  if (cpu_usage > 80 && core_delta > 15) {
    return {
      label: "REPASTE NOW",
      sublabel: "High delta under load — thermal paste degraded",
      color: "#f97316",
      glowColor: "rgba(249,115,22,0.2)",
      severity: "danger",
    };
  }
  if (cpu_usage > 80 && core_delta > 12) {
    return {
      label: "REPASTE SOON",
      sublabel: "Uneven heat spread during heavy loads",
      color: "#d946ef",
      glowColor: "rgba(217,70,239,0.18)",
      severity: "warn",
    };
  }
  if (cpu_usage < 20 && core_delta > 8) {
    return {
      label: "UNEVEN MOUNT",
      sublabel: "Core delta elevated at idle — check cooler seating",
      color: "#f97316",
      glowColor: "rgba(249,115,22,0.18)",
      severity: "warn",
    };
  }
  return {
    label: "SYSTEM HEALTHY",
    sublabel: "Thermal performance within normal range",
    color: "#22d3ee",
    glowColor: "rgba(34,211,238,0.15)",
    severity: "ok",
  };
}

function tempColor(temp: number): string {
  if (temp >= 90) return "#ef4444";
  if (temp >= 80) return "#f97316";
  if (temp >= 70) return "#d946ef";
  if (temp >= 55) return "#84cc16";
  return "#22d3ee";
}

function deltaColor(delta: number): string {
  if (delta >= 15) return "#ef4444";
  if (delta >= 12) return "#f97316";
  if (delta >= 8) return "#d946ef";
  if (delta >= 5) return "#84cc16";
  return "#22d3ee";
}

// ── Shared style tokens ──

const S = {
  sectionHeader: {
    fontSize: 9,
    fontWeight: 600,
    letterSpacing: "0.15em",
    color: "#ced3da80",
  } as const,
  mutedText: {
    fontSize: 9,
    color: "rgba(148,163,184,0.5)",
  } as const,
  brightText: {
    fontSize: 10,
    fontWeight: 700,
    color: "rgba(203,213,225,0.8)",
  } as const,
  cell: {
    background: "rgba(255,255,255,0.04)",
    borderRadius: 8,
  } as const,
} as const;

// ── SVG Ring ──

interface RingProps {
  value: number;
  maxValue?: number;
  radius: number;
  strokeWidth: number;
  color: string;
  bgColor?: string;
  gapDeg?: number;
}

function Ring({ value, maxValue = 100, radius, strokeWidth, color, bgColor = "rgba(255,255,255,0.04)", gapDeg = 80 }: RingProps) {
  const cx = 130;
  const cy = 130;
  const arcDeg = 360 - gapDeg;
  const circumference = 2 * Math.PI * radius;
  const arcLength = (arcDeg / 360) * circumference;
  const fillRatio = Math.min(Math.max(value / maxValue, 0), 1);
  const fillOffset = arcLength * (1 - fillRatio);

  const dashArray = `${arcLength} ${circumference - arcLength}`;
  const rotationDeg = 90 + gapDeg / 2;

  return (
    <g transform={`rotate(${rotationDeg} ${cx} ${cy})`}>
      <circle cx={cx} cy={cy} r={radius} fill="none" stroke={bgColor} strokeWidth={strokeWidth} strokeDasharray={dashArray} strokeLinecap="round" />
      <circle
        cx={cx} cy={cy} r={radius} fill="none" stroke={color} strokeWidth={strokeWidth}
        strokeDasharray={arcLength} strokeDashoffset={fillOffset} strokeLinecap="round"
        style={{ filter: `drop-shadow(0 0 6px ${color}80)` }}
      />
    </g>
  );
}

// ── Core Grid ──

function CoreGrid({ temps }: { temps: number[] }) {
  if (!temps || temps.length === 0) return null;

  return (
    <div style={{ marginTop: 12 }}>
      <div style={{ ...S.sectionHeader, marginBottom: 6 }}>CORE TEMPS</div>
      <div className="grid grid-cols-4 gap-1">
        {temps.map((t, i) => (
          <div
            key={i}
            style={{
              ...S.cell,
              border: `1px solid ${tempColor(t)}30`,
              padding: "5px 4px",
              textAlign: "center",
            }}
          >
            <div style={{ ...S.mutedText, marginBottom: 2 }}>C{i}</div>
            <div style={{ fontSize: 11, fontWeight: 700, color: tempColor(t) }}>{t.toFixed(0)}°</div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ── Metric Bar ──

function MetricBar({ label, value, max, unit, color }: { label: string; value: number; max: number; unit: string; color: string }) {
  const pct = max > 0 ? Math.min((value / max) * 100, 100) : 0;
  return (
    <div style={{ marginBottom: 10 }}>
      <div className="flex justify-between mb-1">
        <span style={{ fontSize: 10, fontWeight: 600, letterSpacing: "0.12em", color: "rgba(148,163,184,0.55)" }}>{label}</span>
        <span style={S.brightText}>
          {value.toFixed(1)} / {max.toFixed(0)} {unit}
        </span>
      </div>
      <div style={{ height: 4, background: "rgba(255,255,255,0.05)", borderRadius: 4, overflow: "hidden" }}>
        <div style={{ height: "100%", width: `${pct}%`, background: color, borderRadius: 4, transition: "width 0.6s ease", boxShadow: `0 0 8px ${color}60` }} />
      </div>
    </div>
  );
}

// ── Process List ──

const processItemStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  padding: "5px 8px",
  background: "rgba(255,255,255,0.03)",
  borderRadius: 8,
  border: "1px solid rgba(255,255,255,0.04)",
};

function formatMem(gb: number): string {
  if (gb >= 1) return `${gb.toFixed(1)}G`;
  const mb = gb * 1024;
  return mb >= 1 ? `${mb.toFixed(0)}M` : `${(mb * 1024).toFixed(0)}K`;
}

function ProcessList({ processes }: { processes: SensorPayload["top_processes"] }) {
  if (!processes || processes.length === 0) {
    return (
      <div>
        <div style={{ ...S.sectionHeader, marginBottom: 8 }}>TOP DEMAND</div>
        <div style={{ fontSize: 10, color: "rgba(148,163,184,0.4)", textAlign: "center", padding: 12 }}>
          No process data available
        </div>
      </div>
    );
  }

  return (
    <div>
      <div style={{ ...S.sectionHeader, marginBottom: 8 }}>TOP DEMAND</div>
      <div className="flex flex-col gap-1">
        {processes.slice(0, 5).map((p, i) => (
          <div key={p.pid} className="animate-fade-in" style={processItemStyle}>
            <div className="flex items-center gap-2 min-w-0">
              <span style={{ fontSize: 9, color: "rgba(100,116,139,0.7)", fontFamily: "monospace", minWidth: 10 }}>{i + 1}</span>
              <span style={{
                fontSize: 11, fontWeight: 500, color: "rgba(203,213,225,0.85)",
                overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 105,
              }}>
                {p.name}
              </span>
            </div>
            <div className="flex gap-2 flex-shrink-0">
              <span style={{ fontSize: 10, fontWeight: 700, color: "#3b82f6", fontFamily: "monospace", minWidth: 38, textAlign: "right" }}>
                {p.cpu_usage.toFixed(1)}%
              </span>
              <span style={{ fontSize: 10, color: "rgba(100,116,139,0.6)", fontFamily: "monospace", minWidth: 36, textAlign: "right" }}>
                {formatMem(p.memory_gb)}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ── Loading State ──

function LoadingState() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-3">
      <svg width="48" height="48" viewBox="0 0 48 48" className="animate-spin-slow">
        <circle cx="24" cy="24" r="20" fill="none" stroke="rgba(34,211,238,0.2)" strokeWidth="2" />
        <circle cx="24" cy="24" r="20" fill="none" stroke="#22d3ee" strokeWidth="2" strokeDasharray="30 96" strokeLinecap="round" />
      </svg>
      <span style={{ fontSize: 11, letterSpacing: "0.15em", color: "rgba(148,163,184,0.4)" }}>INITIALIZING SENSORS</span>
    </div>
  );
}

// ── Insight Badge ──

function InsightBadge({ insight }: { insight: Insight }) {
  return (
    <div
      style={{
        marginTop: 8,
        padding: "6px 16px",
        borderRadius: 99,
        background: insight.glowColor,
        border: `1px solid ${insight.color}40`,
        display: "inline-flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 1,
      }}
    >
      <span style={{ fontSize: 12, fontWeight: 800, color: insight.color, letterSpacing: "0.18em" }}>
        {insight.label}
      </span>
      <span style={{ fontSize: 9, color: "rgba(255, 255, 255, 0.5)", letterSpacing: "0.05em", textAlign: "center" }}>
        {insight.sublabel}
      </span>
    </div>
  );
}

// ── Gauge Section ──

function GaugeSection({ data, insight }: { data: SensorPayload; insight: Insight }) {
  return (
    <div className="flex flex-col items-center relative">
      <div style={{ position: "relative", width: 200, height: 200 }}>
        <svg width="260" height="260" viewBox="0 0 260 260" className="absolute inset-1/2 -translate-x-1/2 -translate-y-1/2">
          <Ring value={data.cpu_pkg_temp} maxValue={100} radius={116} strokeWidth={6} color={tempColor(data.cpu_pkg_temp)} gapDeg={80} />
          <Ring value={data.cpu_usage} maxValue={100} radius={97} strokeWidth={14} color="#3b82f6" gapDeg={80} />
        </svg>

        <div className="absolute inset-0 flex flex-col items-center justify-center gap-0.5">
          <div style={{ fontSize: 36, fontWeight: 900, color: tempColor(data.cpu_pkg_temp), lineHeight: 1, letterSpacing: "-0.02em" }}>
            {data.cpu_pkg_temp > 0 ? `${data.cpu_pkg_temp.toFixed(0)}°` : "--°"}
          </div>
          <div style={{ fontSize: 9, fontWeight: 600, letterSpacing: "0.2em", color: "rgba(255, 255, 255, 0.8)" }}>PKG TEMP</div>

          <div className="flex gap-3.5 mt-1.5">
            <div className="text-center">
              <div style={{ fontSize: 9, color: "rgba(255, 255, 255, 0.8)", letterSpacing: "0.1em" }}>CPU</div>
              <div style={{ fontSize: 13, fontWeight: 700, color: "#3b82f6" }}>{data.cpu_usage.toFixed(1)}%</div>
            </div>
            {data.temps.length >= 2 && (
              <>
                <div style={{ width: 1, background: "rgba(255,255,255,0.06)", height: 24, alignSelf: "center" }} />
                <div className="text-center">
                  <div style={{ fontSize: 9, color: "rgba(255, 255, 255, 0.8)", letterSpacing: "0.1em" }}>Δ</div>
                  <div style={{ fontSize: 13, fontWeight: 700, color: deltaColor(data.core_delta) }}>{data.core_delta.toFixed(1)}°</div>
                </div>
              </>
            )}
          </div>
        </div>
      </div>

      <InsightBadge insight={insight} />
    </div>
  );
}

// ── Divider ──

function Divider() {
  return <div style={{ height: 1, background: "rgba(255,255,255,0.05)" }} />;
}

// ── Main Dashboard ──

export default function Dashboard({ data }: { data: SensorPayload | null }) {
  if (!data) return <LoadingState />;

  const insight = getInsight(data);

  return (
    <ErrorBoundary>
      <div className="flex-1 flex flex-col gap-2.5 min-h-0">
        <GaugeSection data={data} insight={insight} />
        <CoreGrid temps={data.temps} />
        <Divider />

        <div>
          <div style={{ ...S.sectionHeader, marginBottom: 8 }}>SYSTEM</div>
          <MetricBar label="RAM" value={data.ram_usage} max={data.ram_total} unit="GB" color="#8b5cf6" />
          <MetricBar label="DISK" value={data.disk_usage} max={data.disk_total} unit="GB" color="#06b6d4" />
        </div>

        <Divider />
        <div className="flex-1 min-h-0">
          <ProcessList processes={data.top_processes} />
        </div>
      </div>
    </ErrorBoundary>
  );
}