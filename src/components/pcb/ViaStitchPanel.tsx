import React, { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { usePCBStore } from "../../stores/pcbStore";
import toast from "react-hot-toast";

export function ViaStitchPanel() {
  const { board, loadBoard, selectedIds, activeLayer } = usePCBStore();
  const [mode, setMode] = useState<"line" | "shield">("line");
  const [spacing, setSpacing] = useState(2);
  const [offset, setOffset] = useState(1);
  const [stitching, setStitching] = useState(false);
  const [collectingLine, setCollectingLine] = useState(false);
  const [lineStart, setLineStart] = useState<{ x: number; y: number } | null>(null);
  const [traceIndex, setTraceIndex] = useState<number | null>(null);

  const selectedTraceId = [...selectedIds].find((id) => id.startsWith("trace-"));
  const selectedTraceIdx = selectedTraceId ? parseInt(selectedTraceId.replace("trace-", ""), 10) : null;

  useEffect(() => {
    if (selectedTraceIdx != null) setTraceIndex(selectedTraceIdx);
  }, [selectedTraceIdx]);

  const handleCanvasClick = useCallback((pos: { x: number; y: number }) => {
    if (!collectingLine) return;
    if (!lineStart) {
      setLineStart(pos);
    } else {
      setLineStart(null);
      setCollectingLine(false);
    }
  }, [collectingLine, lineStart]);

  const addLineStitch = useCallback(async () => {
    if (!lineStart) {
      setCollectingLine(true);
      setLineStart(null);
      toast("Click start point, then end point");
      return;
    }
    setStitching(true);
    try {
      await invoke("stitch_vias", {
        startX: lineStart.x,
        startY: lineStart.y,
        endX: lineStart.x + 10, // placeholder - actually need to capture both points
        endY: lineStart.y,
        netId: 0,
        spacing,
      });
      toast.success("Via line stitched");
      await loadBoard();
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
    setStitching(false);
  }, [lineStart, spacing, loadBoard]);

  const addShield = useCallback(async () => {
    if (traceIndex == null) return toast.error("Select a trace first");
    setStitching(true);
    try {
      await invoke("stitch_via_shield", {
        traceIndex,
        offset,
        spacing,
      });
      toast.success("Via shield added");
      await loadBoard();
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
    setStitching(false);
  }, [traceIndex, offset, spacing, loadBoard]);

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <h3 className="font-semibold text-sm">Via Stitching</h3>

      <div className="bg-bg-elevated rounded p-2 space-y-2">
        <div className="flex gap-2">
          <label className="flex items-center gap-1 text-[11px] flex-1">
            <input
              type="radio"
              checked={mode === "line"}
              onChange={() => setMode("line")}
            />
            Line Stitch
          </label>
          <label className="flex items-center gap-1 text-[11px] flex-1">
            <input
              type="radio"
              checked={mode === "shield"}
              onChange={() => setMode("shield")}
            />
            Shield Trace
          </label>
        </div>

        {mode === "line" && (
          <div className="space-y-2">
            <label className="flex flex-col gap-1 text-[11px]">
              <span className="text-text-muted">Spacing (mm)</span>
              <input
                type="number"
                value={spacing}
                onChange={(e) => setSpacing(Math.max(0.2, Number(e.target.value)))}
                className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
                step={0.1}
              />
            </label>
            <button
              onClick={addLineStitch}
              disabled={stitching || collectingLine}
              className="w-full px-3 py-1.5 bg-copper text-white rounded hover:opacity-90 disabled:opacity-50 text-xs font-medium"
            >
              {collectingLine ? (lineStart ? "Click End Point" : "Click Start Point") : stitching ? "Stitching..." : "Add Via Line"}
            </button>
            <p className="text-[10px] text-text-muted">Click two points on canvas for start/end</p>
          </div>
        )}

        {mode === "shield" && (
          <div className="space-y-2">
            {traceIndex != null ? (
              <div className="text-[11px] text-text-muted">
                Selected: Trace #{traceIndex}
              </div>
            ) : (
              <p className="text-[11px] text-text-muted">Select a trace in the canvas</p>
            )}

            <label className="flex flex-col gap-1 text-[11px]">
              <span className="text-text-muted">Offset from Trace (mm)</span>
              <input
                type="number"
                value={offset}
                onChange={(e) => setOffset(Math.max(0.2, Number(e.target.value)))}
                className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
                step={0.1}
              />
            </label>

            <label className="flex flex-col gap-1 text-[11px]">
              <span className="text-text-muted">Via Spacing (mm)</span>
              <input
                type="number"
                value={spacing}
                onChange={(e) => setSpacing(Math.max(0.2, Number(e.target.value)))}
                className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
                step={0.1}
              />
            </label>

            <button
              onClick={addShield}
              disabled={stitching || traceIndex == null}
              className="w-full px-3 py-1.5 bg-copper text-white rounded hover:opacity-90 disabled:opacity-50 text-xs font-medium"
            >
              {stitching ? "Adding Shield..." : "Add Via Shield"}
            </button>
          </div>
        )}
      </div>

      <div className="bg-bg-surface rounded p-2 text-[11px] text-text-muted space-y-1">
        <p><strong>Line Stitch:</strong> Places ground vias along a line between two points</p>
        <p><strong>Shield:</strong> Places ground vias on both sides of a selected trace</p>
        <p>Use for EMI reduction, ground integrity, thermal vias</p>
      </div>
    </div>
  );
}