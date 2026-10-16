import React, { useState, useEffect, useCallback } from "react";
import { usePCBStore } from "../../stores/pcbStore";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";

interface SiResult {
  trace_length_mm: number;
  impedance_ohms: number;
  propagation_delay_ps: number;
  reflection_coefficient: number;
  return_loss_db: number;
  dominant_freq_ghz: number;
  attenuation_db_per_m: number;
}

interface CrosstalkResult {
  victim_trace: number;
  aggressor_trace: number;
  coupling_length_mm: number;
  near_end_crosstalk_db: number;
  far_end_crosstalk_db: number;
}

export function SIPanel() {
  const { board, selectedIds } = usePCBStore();
  const [results, setResults] = useState<SiResult[] | null>(null);
  const [crosstalk, setCrosstalk] = useState<CrosstalkResult[] | null>(null);
  const [analyzing, setAnalyzing] = useState(false);

  const selectedTraceId = [...selectedIds].find((id) => id.startsWith("trace-"));
  const traceIndex = selectedTraceId != null ? parseInt(selectedTraceId.replace("trace-", ""), 10) : null;

  const runAnalysis = useCallback(async () => {
    if (!board) return;
    setAnalyzing(true);
    try {
      const si = await invoke<SiResult[]>("analyze_signal_integrity", { traceIndex });
      setResults(si);
      const xt = await invoke<CrosstalkResult[]>("analyze_crosstalk");
      setCrosstalk(xt);
    } catch (e: any) {
      toast.error(`SI analysis failed: ${e}`);
    }
    setAnalyzing(false);
  }, [board, traceIndex]);

  useEffect(() => {
    setResults(null);
    setCrosstalk(null);
  }, [board]);

  const selectedResult = traceIndex != null && results ? results.find((_, i) => i === traceIndex) : null;

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <h3 className="font-semibold text-sm">Signal Integrity</h3>

      <button
        onClick={runAnalysis}
        disabled={analyzing}
        className="w-full px-3 py-1.5 bg-copper text-white rounded hover:opacity-90 disabled:opacity-50 text-xs font-medium"
      >
        {analyzing ? "Analyzing..." : traceIndex != null ? "Analyze Selected Trace" : "Analyze All Traces"}
      </button>

      {selectedResult && (
        <div className="bg-bg-elevated rounded p-2 space-y-1">
          <div className="font-medium text-copper">Trace #{traceIndex}</div>
          <Row label="Length" value={`${selectedResult.trace_length_mm.toFixed(2)} mm`} />
          <Row label="Impedance (Z₀)" value={`${selectedResult.impedance_ohms.toFixed(1)} Ω`} />
          <Row label="Prop. Delay" value={`${selectedResult.propagation_delay_ps.toFixed(1)} ps`} />
          <Row label="Reflection (Γ)" value={selectedResult.reflection_coefficient.toFixed(4)} />
          <Row label="Return Loss" value={`${selectedResult.return_loss_db.toFixed(1)} dB`} />
          <Row label="Resonant Freq" value={`${selectedResult.dominant_freq_ghz.toFixed(2)} GHz`} />
          <Row label="Attenuation" value={`${selectedResult.attenuation_db_per_m.toFixed(2)} dB/m`} />
        </div>
      )}

      {!selectedResult && results && results.length > 0 && (
        <div className="bg-bg-elevated rounded p-2 space-y-1 max-h-48 overflow-y-auto">
          <div className="font-medium text-copper">All Traces ({results.length})</div>
          {results.map((r, i) => (
            <div key={i} className="flex justify-between text-[11px] border-b border-border pb-0.5">
              <span>Trace #{i}</span>
              <span>{r.impedance_ohms.toFixed(0)} Ω / {r.trace_length_mm.toFixed(1)} mm</span>
            </div>
          ))}
        </div>
      )}

      {results && results.length === 0 && (
        <p className="text-text-muted italic">No traces to analyze.</p>
      )}

      {crosstalk && crosstalk.length > 0 && (
        <div className="space-y-1">
          <h4 className="font-medium text-copper">Crosstalk</h4>
          {crosstalk.map((x, i) => (
            <div key={i} className="bg-bg-elevated rounded p-2 text-[11px] space-y-0.5">
              <div className="font-medium">T{x.victim_trace} ← T{x.aggressor_trace}</div>
              <Row label="Coupling" value={`${x.coupling_length_mm.toFixed(1)} mm`} />
              <Row label="NEXT" value={`${x.near_end_crosstalk_db.toFixed(1)} dB`} />
              <Row label="FEXT" value={`${x.far_end_crosstalk_db.toFixed(1)} dB`} />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between">
      <span className="text-text-muted">{label}</span>
      <span className="font-mono">{value}</span>
    </div>
  );
}
