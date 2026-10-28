import React, { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Play, Download, AlertTriangle } from "lucide-react";

interface SpicePlot {
  plot_name: string;
  variables: string[];
  points: number;
  data: number[][];
}

interface SpiceSimResult {
  plots: SpicePlot[];
  stdout: string;
  netlist: string;
}

const PRESETS: { id: string; label: string; directive: string }[] = [
  { id: "tran", label: "Transient", directive: ".tran 1m 100m" },
  { id: "ac", label: "AC Sweep", directive: ".ac dec 20 1 100Meg" },
  { id: "op", label: "Operating Point", directive: ".op" },
  { id: "dc", label: "DC Sweep", directive: ".dc V1 0 5 0.1" },
];

const TRACE_COLORS = ["#D4783C", "#5096E0", "#6BCB77", "#E0C050"];

export function SimulationPanel() {
  const [analysis, setAnalysis] = useState(PRESETS[0].directive);
  const [result, setResult] = useState<SpiceSimResult | null>(null);
  const [activePlot, setActivePlot] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  const run = async () => {
    setRunning(true);
    setError(null);
    try {
      const res = await invoke<SpiceSimResult>("simulate_schematic", { analysis });
      setResult(res);
      setActivePlot(res.plots.length - 1);
    } catch (e: any) {
      setResult(null);
      setError(String(e));
    }
    setRunning(false);
  };

  const exportNetlist = async () => {
    try {
      const path = await save({
        filters: [{ name: "SPICE Netlist", extensions: ["cir", "sp"] }],
        defaultPath: "tracecraft.cir",
      });
      if (path) {
        await invoke("export_spice_netlist", { path });
      }
    } catch (e: any) {
      setError(String(e));
    }
  };

  const plot = result?.plots[activePlot];

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <h3 className="font-semibold text-sm">Circuit Simulation</h3>
      <p className="text-[11px] text-text-muted">
        Runs the current schematic through ngspice. Requires the <code>ngspice</code> binary on your PATH.
      </p>

      <div className="bg-bg-elevated rounded p-2 space-y-2">
        <label className="flex flex-col gap-1 text-[11px]">
          <span className="text-text-muted">Analysis preset</span>
          <select
            value={PRESETS.find((p) => p.directive === analysis)?.id ?? "custom"}
            onChange={(e) => {
              const preset = PRESETS.find((p) => p.id === e.target.value);
              if (preset) setAnalysis(preset.directive);
            }}
            className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
          >
            {PRESETS.map((p) => (
              <option key={p.id} value={p.id}>{p.label}</option>
            ))}
            <option value="custom">Custom</option>
          </select>
        </label>

        <label className="flex flex-col gap-1 text-[11px]">
          <span className="text-text-muted">SPICE directive</span>
          <input
            value={analysis}
            onChange={(e) => setAnalysis(e.target.value)}
            spellCheck={false}
            className="px-2 py-1 bg-bg-surface border border-border rounded text-xs font-mono"
          />
        </label>

        <div className="flex gap-2">
          <button
            onClick={run}
            disabled={running}
            className="flex items-center justify-center gap-1 flex-1 px-2 py-1.5 rounded bg-copper text-white hover:bg-copper-dark disabled:opacity-50"
          >
            <Play size={12} /> {running ? "Running..." : "Run"}
          </button>
          <button
            onClick={exportNetlist}
            title="Export netlist"
            className="flex items-center justify-center gap-1 px-2 py-1.5 rounded bg-bg-surface border border-border text-text hover:bg-bg-elevated"
          >
            <Download size={12} /> .cir
          </button>
        </div>
      </div>

      {error && (
        <div className="rounded border border-red-500/40 bg-red-500/10 p-2 text-[11px] text-red-300 flex gap-1.5">
          <AlertTriangle size={13} className="shrink-0 mt-0.5" />
          <pre className="whitespace-pre-wrap break-words font-mono">{error}</pre>
        </div>
      )}

      {plot && (
        <>
          {result!.plots.length > 1 && (
            <select
              value={activePlot}
              onChange={(e) => setActivePlot(Number(e.target.value))}
              className="w-full px-2 py-1 bg-bg-surface border border-border rounded text-xs"
            >
              {result!.plots.map((p, i) => (
                <option key={i} value={i}>{p.plot_name} ({p.points} pts)</option>
              ))}
            </select>
          )}
          <PlotChart plot={plot} />
          <div className="text-[10px] text-text-muted">
            {plot.plot_name} · {plot.points} points · {plot.variables.length} vectors
          </div>
        </>
      )}

      {result && (
        <details className="bg-bg-elevated rounded p-2">
          <summary className="cursor-pointer text-[11px] text-text-muted">Generated netlist</summary>
          <pre className="mt-2 max-h-56 overflow-auto text-[10px] font-mono whitespace-pre text-text-secondary">
            {result.netlist}
          </pre>
        </details>
      )}
    </div>
  );
}

function PlotChart({ plot }: { plot: SpicePlot }) {
  const width = 288;
  const height = 170;
  const pad = { left: 34, right: 8, top: 8, bottom: 18 };

  const series = useMemo(() => {
    const x = plot.data[0] ?? [];
    return plot.data
      .slice(1, 5)
      .map((values, i) => ({
        name: plot.variables[i + 1] ?? `v${i + 1}`,
        color: TRACE_COLORS[i % TRACE_COLORS.length],
        values,
      }));
  }, [plot]);

  const xMin = Math.min(...(plot.data[0] ?? [0]));
  const xMax = Math.max(...(plot.data[0] ?? [1]));
  const allY = series.flatMap((s) => s.values);
  let yMin = Math.min(...allY);
  let yMax = Math.max(...allY);
  if (!isFinite(yMin) || !isFinite(yMax)) {
    yMin = 0;
    yMax = 1;
  }
  if (yMax - yMin < 1e-12) {
    yMin -= 0.5;
    yMax += 0.5;
  }
  const spanX = xMax - xMin || 1;
  const spanY = yMax - yMin;

  const sx = (v: number) => pad.left + ((v - xMin) / spanX) * (width - pad.left - pad.right);
  const sy = (v: number) => height - pad.bottom - ((v - yMin) / spanY) * (height - pad.top - pad.bottom);

  const gridLines = [0, 0.25, 0.5, 0.75, 1];

  return (
    <div className="bg-bg-surface border border-border rounded p-1">
      <svg width="100%" viewBox={`0 0 ${width} ${height}`} className="block">
        {gridLines.map((f) => (
          <line
            key={f}
            x1={pad.left}
            x2={width - pad.right}
            y1={pad.top + f * (height - pad.top - pad.bottom)}
            y2={pad.top + f * (height - pad.top - pad.bottom)}
            stroke="#2D2F52"
            strokeWidth={1}
          />
        ))}
        <line x1={pad.left} x2={pad.left} y1={pad.top} y2={height - pad.bottom} stroke="#5C5E7E" />
        <line x1={pad.left} x2={width - pad.right} y1={height - pad.bottom} y2={height - pad.bottom} stroke="#5C5E7E" />

        {series.map((s) => {
          const points = s.values
            .map((v, i) => `${sx(plot.data[0][i] ?? 0).toFixed(2)},${sy(v).toFixed(2)}`)
            .join(" ");
          return <polyline key={s.name} points={points} fill="none" stroke={s.color} strokeWidth={1.4} />;
        })}

        <text x={pad.left - 3} y={pad.top + 4} textAnchor="end" fontSize={7} fill="#9496B8">
          {formatNumber(yMax)}
        </text>
        <text x={pad.left - 3} y={height - pad.bottom} textAnchor="end" fontSize={7} fill="#9496B8">
          {formatNumber(yMin)}
        </text>
        <text x={width - pad.right} y={height - 5} textAnchor="end" fontSize={7} fill="#9496B8">
          {formatNumber(xMax)}
        </text>
        <text x={pad.left} y={height - 5} textAnchor="start" fontSize={7} fill="#9496B8">
          {formatNumber(xMin)}
        </text>
        <text x={pad.left} y={height - pad.bottom + 12} fontSize={7} fill="#9496B8">
          {plot.variables[0] ?? "x"}
        </text>
      </svg>
      <div className="flex flex-wrap gap-x-3 gap-y-1 px-1 pb-1">
        {series.map((s) => (
          <span key={s.name} className="flex items-center gap-1 text-[10px] text-text-secondary">
            <span className="inline-block w-2 h-2 rounded-sm" style={{ background: s.color }} />
            {s.name}
          </span>
        ))}
      </div>
    </div>
  );
}

function formatNumber(v: number): string {
  if (!isFinite(v)) return "—";
  const abs = Math.abs(v);
  if (abs !== 0 && (abs >= 1e4 || abs < 1e-3)) return v.toExponential(1);
  return v.toFixed(3).replace(/\.?0+$/, "") || "0";
}
