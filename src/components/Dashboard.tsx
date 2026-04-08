import { SensorPayload } from "../App";

// Thermal Intelligence 
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

// SVG Ring Component 
interface RingProps {
  value: number;       // 0–100
  maxValue?: number;
  radius: number;
  strokeWidth: number;
  color: string;
  bgColor?: string;
  gapDeg?: number;     // gap at the bottom in degrees (default 80)
}

function Ring({ value, maxValue = 100, radius, strokeWidth, color, bgColor = "rgba(255,255,255,0.04)", gapDeg = 80 }: RingProps) {
  const cx = 130;
  const cy = 130;
  const arcDeg = 360 - gapDeg;
  const circumference = 2 * Math.PI * radius;
  const arcLength = (arcDeg / 360) * circumference;
  const fillLength = Math.min(Math.max((value / maxValue), 0), 1) * arcLength;

  // strokeDasharray: draw the arc portion then transparent gap
  const dashArray = `${arcLength} ${circumference - arcLength}`;
  const fillDashArray = `${fillLength} ${circumference - fillLength}`;

  // Rotate so the gap sits at the bottom: start at top-left of gap
  const rotationDeg = 90 + gapDeg / 2;

  return (
    <g transform={`rotate(${rotationDeg} ${cx} ${cy})`}>
      {/* Background track */}
      <circle
        cx={cx} cy={cy} r={radius}
        fill="none"
        stroke={bgColor}
        strokeWidth={strokeWidth}
        strokeDasharray={dashArray}
        strokeLinecap="round"
      />
      {/* Value fill */}
      <circle
        cx={cx} cy={cy} r={radius}
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
        strokeDasharray={fillDashArray}
        strokeLinecap="round"
        style={{ filter: `drop-shadow(0 0 6px ${color}80)` }}
      />
    </g>
  );
}

// Core Grid
function CoreGrid({ temps }: { temps: number[] }) {
  if (!temps || temps.length === 0) return null;

  return (
    <div style={{ marginTop: 12 }}>
      <div style={{ fontSize: 9, fontWeight: 600, letterSpacing: "0.15em", color: "#ced3da80", marginBottom: 6 }}>
        CORE TEMPS
      </div>
      <div style={{ display: "grid", gridTemplateColumns: `repeat(${Math.min(temps.length, 4)}, 1fr)`, gap: 4 }}>
        {temps.map((t, i) => (
          <div
            key={i}
            style={{
              background: "rgba(255,255,255,0.04)",
              border: `1px solid ${tempColor(t)}30`,
              borderRadius: 8,
              padding: "5px 4px",
              textAlign: "center",
            }}
          >
            <div style={{ fontSize: 9, color: "rgba(148,163,184,0.5)", marginBottom: 2 }}>C{i}</div>
            <div style={{ fontSize: 11, fontWeight: 700, color: tempColor(t) }}>{t.toFixed(0)}°</div>
          </div>
        ))}
      </div>
    </div>
  );
}

// Metric Bar
function MetricBar({ label, value, max, unit, color }: { label: string; value: number; max: number; unit: string; color: string }) {
  const pct = Math.min((value / max) * 100, 100);
  return (
    <div style={{ marginBottom: 10 }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 4 }}>
        <span style={{ fontSize: 10, fontWeight: 600, letterSpacing: "0.12em", color: "rgba(148,163,184,0.55)" }}>{label}</span>
        <span style={{ fontSize: 10, fontWeight: 700, color: "rgba(203,213,225,0.8)" }}>
          {value.toFixed(1)} / {max.toFixed(0)} {unit}
        </span>
      </div>
      <div style={{ height: 4, background: "rgba(255,255,255,0.05)", borderRadius: 4, overflow: "hidden" }}>
        <div
          style={{
            height: "100%",
            width: `${pct}%`,
            background: color,
            borderRadius: 4,
            transition: "width 0.6s ease",
            boxShadow: `0 0 8px ${color}60`,
          }}
        />
      </div>
    </div>
  );
}

// Process List
function ProcessList({ processes }: { processes: SensorPayload["top_processes"] }) {
  return (
    <div>
      <div style={{ fontSize: 9, fontWeight: 600, letterSpacing: "0.15em", color: "#ced3da80", marginBottom: 8 }}>
        TOP DEMAND
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
        {processes.map((p, i) => (
          <div
            key={p.pid}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "5px 8px",
              background: "rgba(255,255,255,0.03)",
              borderRadius: 8,
              border: "1px solid rgba(255,255,255,0.04)",
            }}
            className="animate-fade-in"
          >
            <div style={{ display: "flex", alignItems: "center", gap: 8, minWidth: 0 }}>
              <span style={{ fontSize: 9, color: "rgba(100,116,139,0.7)", fontFamily: "monospace", minWidth: 10 }}>{i + 1}</span>
              <span style={{ fontSize: 11, fontWeight: 500, color: "rgba(203,213,225,0.85)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 105 }}>
                {p.name}
              </span>
            </div>
            <div style={{ display: "flex", gap: 8, flexShrink: 0 }}>
              <span style={{ fontSize: 10, fontWeight: 700, color: "#3b82f6", fontFamily: "monospace", minWidth: 38, textAlign: "right" }}>
                {p.cpu_usage.toFixed(1)}%
              </span>
              <span style={{ fontSize: 10, color: "rgba(100,116,139,0.6)", fontFamily: "monospace", minWidth: 36, textAlign: "right" }}>
                {p.memory_mb < 1024 ? `${p.memory_mb.toFixed(0)}M` : `${(p.memory_mb / 1024).toFixed(1)}G`}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// Main Dashboard
export default function Dashboard({ data }: { data: SensorPayload | null }) {
  if (!data) {
    return (
      <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: 12 }}>
        <div style={{ position: "relative", width: 48, height: 48 }}>
          <svg width="48" height="48" viewBox="0 0 48 48" style={{ animation: "spin-slow 3s linear infinite" }}>
            <circle cx="24" cy="24" r="20" fill="none" stroke="rgba(34,211,238,0.2)" strokeWidth="2" />
            <circle cx="24" cy="24" r="20" fill="none" stroke="#22d3ee" strokeWidth="2"
              strokeDasharray="30 96" strokeLinecap="round" />
          </svg>
        </div>
        <span style={{ fontSize: 11, letterSpacing: "0.15em", color: "rgba(148,163,184,0.4)" }}>INITIALIZING SENSORS</span>
      </div>
    );
  }

  const insight = getInsight(data);

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 10, minHeight: 0 }}>

      {/* ── Gauge Section ── */}
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", position: "relative" }}>
        <div style={{ position: "relative", width: 200, height: 200 }}>
          <svg width="260" height="260" viewBox="0 0 260 260" style={{ position: "absolute", top: "50%", left: "50%", transform: "translate(-50%, -50%)" }}>
            {/* Outer: Temperature ring */}
            <Ring
              value={data.cpu_pkg_temp}
              maxValue={100}
              radius={116}
              strokeWidth={6}
              color={tempColor(data.cpu_pkg_temp)}
              gapDeg={80}
            />
            {/* Middle: CPU Usage ring */}
            <Ring
              value={data.cpu_usage}
              maxValue={100}
              radius={97}
              strokeWidth={14}
              color="#3b82f6"
              gapDeg={80}
            />
          </svg>

          {/* Center text */}
          <div style={{
            position: "absolute", inset: 0,
            display: "flex", flexDirection: "column",
            alignItems: "center", justifyContent: "center",
            gap: 2
          }}>
            <div style={{ fontSize: 36, fontWeight: 900, color: tempColor(data.cpu_pkg_temp), lineHeight: 1, letterSpacing: "-0.02em" }}>
              {data.cpu_pkg_temp > 0 ? `${data.cpu_pkg_temp.toFixed(0)}°` : "--°"}
            </div>
            <div style={{ fontSize: 9, fontWeight: 600, letterSpacing: "0.2em", color: "rgba(255, 255, 255, 0.8)" }}>PKG TEMP</div>

            <div style={{ display: "flex", gap: 14, marginTop: 6 }}>
              <div style={{ textAlign: "center" }}>
                <div style={{ fontSize: 9, color: "rgba(255, 255, 255, 0.8)", letterSpacing: "0.1em" }}>CPU</div>
                <div style={{ fontSize: 13, fontWeight: 700, color: "#3b82f6" }}>{data.cpu_usage.toFixed(1)}%</div>
              </div>
              <div style={{ width: 1, background: "rgba(255,255,255,0.06)", height: 24, alignSelf: "center" }} />
              <div style={{ textAlign: "center" }}>
                <div style={{ fontSize: 9, color: "rgba(255, 255, 255, 0.8)", letterSpacing: "0.1em" }}>Δ</div>
                <div style={{ fontSize: 13, fontWeight: 700, color: tempColor(data.cpu_pkg_temp - data.core_delta) }}>{data.core_delta.toFixed(1)}°</div>
              </div>
            </div>
          </div>
        </div>

        {/* Insight badge */}
        <div
          style={{
            marginTop: "8px",
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
      </div>

      {/* ── Core temps ── */}
      <CoreGrid temps={data.temps} />

      {/* ── Divider ── */}
      <div style={{ height: 1, background: "rgba(255,255,255,0.05)" }} />

      {/* ── System metrics ── */}
      <div>
        <div style={{ fontSize: 9, fontWeight: 600, letterSpacing: "0.15em", color: "#ced3da80", marginBottom: 8 }}>
          SYSTEM
        </div>
        <MetricBar label="RAM" value={data.ram_usage / 1024} max={data.ram_total / 1024} unit="GB" color="#8b5cf6" />
        <MetricBar label="DISK" value={data.disk_usage} max={data.disk_total} unit="GB" color="#06b6d4" />
      </div>

      {/* ── Divider ── */}
      <div style={{ height: 1, background: "rgba(255,255,255,0.05)" }} />

      {/* ── Process list ── */}
      <div style={{ flex: 1, minHeight: 0 }}>
        <ProcessList processes={data.top_processes} />
      </div>
    </div>
  );
}
