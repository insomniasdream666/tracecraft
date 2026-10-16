import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { usePCBStore } from "../../stores/pcbStore";
import toast from "react-hot-toast";

interface LayerDef {
  id: number;
  name: string;
  kind: string;
  color: string;
}

interface Dielectric {
  name: string;
  er: number;
  thickness_mm: number;
  loss_tangent: number;
}

interface ImpedanceResult {
  trace_width_mm: number;
  impedance_ohms: number;
  trace_thickness_mm: number;
}

interface LayerStack {
  layers: LayerDef[];
  dielectrics: Dielectric[];
  copper_weight_oz: number;
}

export function ImpedancePanel() {
  const { board, loadBoard } = usePCBStore();
  const [stackup, setStackup] = useState<LayerStack | null>(null);
  const [targetImpedance, setTargetImpedance] = useState(50);
  const [selectedLayer, setSelectedLayer] = useState<number>(1);
  const [result, setResult] = useState<ImpedanceResult | null>(null);
  const [calculating, setCalculating] = useState(false);

  useEffect(() => {
    if (board?.layers) {
      setStackup(board.layers);
      const signalLayer = board.layers.layers.find((l: LayerDef) => l.kind === "Top" || l.kind === "Bottom" || l.kind === "Inner");
      if (signalLayer) setSelectedLayer(signalLayer.id);
    }
  }, [board]);

  const computeImpedance = useCallback(async () => {
    if (!stackup) return;
    setCalculating(true);
    try {
      const res = await invoke<ImpedanceResult>("compute_impedance", {
        targetImpedance,
        layerIndex: stackup.layers.findIndex((l: LayerDef) => l.id === selectedLayer),
        copperWeightOz: stackup.copper_weight_oz,
      });
      setResult(res);
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
    setCalculating(false);
  }, [stackup, targetImpedance, selectedLayer]);

  useEffect(() => {
    computeImpedance();
  }, [computeImpedance]);

  if (!stackup) return <div className="p-3 text-xs text-text-muted">No board loaded</div>;

  const signalLayers = stackup.layers.filter((l: LayerDef) => l.kind === "Top" || l.kind === "Bottom" || l.kind === "Inner");

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <h3 className="font-semibold text-sm">Impedance Calculator</h3>

      <div className="bg-bg-elevated rounded p-2 space-y-2">
        <label className="flex flex-col gap-1 text-[11px]">
          <span className="text-text-muted">Target Z₀</span>
          <input
            type="number"
            value={targetImpedance}
            onChange={(e) => setTargetImpedance(Number(e.target.value))}
            className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
            step={1}
            min={10}
            max={150}
          />
          <span className="text-text-muted">Ω</span>
        </label>

        <label className="flex flex-col gap-1 text-[11px]">
          <span className="text-text-muted">Layer</span>
          <select
            value={selectedLayer}
            onChange={(e) => setSelectedLayer(Number(e.target.value))}
            className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
          >
            {signalLayers.map((l: LayerDef) => (
              <option key={l.id} value={l.id}>{l.name} (Layer {l.id})</option>
            ))}
          </select>
        </label>

        {result && (
          <div className="bg-bg-surface rounded p-2 space-y-1 border border-copper/30">
            <div className="font-medium text-copper">Result</div>
            <Row label="Trace Width" value={`${result.trace_width_mm.toFixed(3)} mm`} />
            <Row label="Calculated Z₀" value={`${result.impedance_ohms.toFixed(2)} Ω`} />
            <Row label="Error" value={`${(result.impedance_ohms - targetImpedance).toFixed(2)} Ω`} />
            <Row label="Copper Weight" value={`${result.trace_thickness_mm * 1000} µm`} />
          </div>
        )}
      </div>

      <h4 className="font-medium text-copper">Stackup</h4>
      <div className="bg-bg-elevated rounded p-2 space-y-1 max-h-48 overflow-y-auto">
        {stackup.layers.map((l: LayerDef) => (
          <div key={l.id} className="flex justify-between text-[11px]">
            <span className="flex items-center gap-1">
              <span style={{ width: 8, height: 8, backgroundColor: l.color, borderRadius: 2, display: "inline-block" }} />
              {l.name} ({l.kind})
            </span>
            <span>{l.id}</span>
          </div>
        ))}
        {stackup.dielectrics.map((d: Dielectric, i: number) => (
          <div key={i} className="flex justify-between text-[11px] text-text-muted pl-4">
            <span>📦 {d.name} (Er={d.er})</span>
            <span>{d.thickness_mm} mm</span>
          </div>
        ))}
        <div className="flex justify-between text-[11px] text-text-muted pt-1 border-t border-border">
          <span>Copper Weight</span>
          <span>{stackup.copper_weight_oz} oz</span>
        </div>
      </div>

      <button
        onClick={async () => {
          try {
            await invoke("set_stackup", { stackup });
            toast.success("Stackup saved");
            await loadBoard();
          } catch (e: any) {
            toast.error(`Failed: ${e}`);
          }
        }}
        className="w-full px-3 py-1.5 bg-copper text-white rounded hover:opacity-90 text-xs font-medium"
      >
        Save Stackup
      </button>
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