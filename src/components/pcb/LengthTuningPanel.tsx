import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { usePCBStore } from "../../stores/pcbStore";
import toast from "react-hot-toast";

export function LengthTuningPanel() {
  const { board, selectedIds, loadBoard } = usePCBStore();
  const [extraLength, setExtraLength] = useState(5);
  const [segmentIndex, setSegmentIndex] = useState(0);
  const [tuning, setTuning] = useState(false);
  const [traceInfo, setTraceInfo] = useState<{ id: number; points: number; segments: number } | null>(null);

  const selectedTraceId = [...selectedIds].find((id) => id.startsWith("trace-"));
  const traceId = selectedTraceId ? parseInt(selectedTraceId.replace("trace-", ""), 10) : null;

  useEffect(() => {
    if (traceId != null && board?.traces[traceId]) {
      const trace = board.traces[traceId];
      setTraceInfo({
        id: traceId,
        points: trace.points.length,
        segments: Math.max(0, trace.points.length - 1),
      });
      if (segmentIndex >= trace.points.length - 1) {
        setSegmentIndex(Math.max(0, trace.points.length - 2));
      }
    } else {
      setTraceInfo(null);
    }
  }, [traceId, board, segmentIndex]);

  const applyTuning = useCallback(async () => {
    if (traceId == null) return;
    setTuning(true);
    try {
      await invoke("apply_length_tuning", {
        traceIndex: traceId,
        segmentIndex,
        extraLength,
      });
      toast.success(`Added ${extraLength}mm serpentine to trace #${traceId}`);
      await loadBoard();
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
    setTuning(false);
  }, [traceId, segmentIndex, extraLength, loadBoard]);

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <h3 className="font-semibold text-sm">Length Tuning</h3>

      {traceInfo ? (
        <div className="bg-bg-elevated rounded p-2 space-y-2">
          <div className="font-medium">Trace #{traceInfo.id}</div>
          <Row label="Points" value={traceInfo.points.toString()} />
          <Row label="Segments" value={traceInfo.segments.toString()} />

          <label className="flex flex-col gap-1 text-[11px]">
            <span className="text-text-muted">Segment Index</span>
            <input
              type="number"
              value={segmentIndex}
              onChange={(e) => setSegmentIndex(Math.max(0, Math.min(traceInfo.segments - 1, Number(e.target.value))))}
              className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
              min={0}
              max={Math.max(0, traceInfo.segments - 1)}
            />
          </label>

          <label className="flex flex-col gap-1 text-[11px]">
            <span className="text-text-muted">Extra Length (mm)</span>
            <input
              type="number"
              value={extraLength}
              onChange={(e) => setExtraLength(Math.max(0.1, Number(e.target.value)))}
              className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
              step={0.1}
              min={0.1}
              max={100}
            />
          </label>

          <button
            onClick={applyTuning}
            disabled={tuning}
            className="w-full px-3 py-1.5 bg-copper text-white rounded hover:opacity-90 disabled:opacity-50 text-xs font-medium"
          >
            {tuning ? "Applying..." : "Apply Serpentine Tuning"}
          </button>
        </div>
      ) : (
        <div className="bg-bg-elevated rounded p-2 text-center text-text-muted">
          Select a trace to apply length tuning
        </div>
      )}

      <div className="bg-bg-surface rounded p-2 text-[11px] text-text-muted space-y-1">
        <p>Adds serpentine (accordion) pattern to selected trace segment</p>
        <p>Use to match trace lengths for differential pairs / buses</p>
      </div>
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