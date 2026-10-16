import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { usePCBStore } from "../../stores/pcbStore";
import { useUIStore } from "../../stores/uiStore";
import { FlexZone } from "../../types/pcb";
import { Point } from "../../lib/geometry";
import toast from "react-hot-toast";

export function FlexZonePanel() {
  const { board, loadBoard } = usePCBStore();
  const { activeTool, setTool } = useUIStore();
  const [flexZones, setFlexZones] = useState<FlexZone[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [bendRadius, setBendRadius] = useState(5);
  const [numBends, setNumBends] = useState(1);
  const [drawingMode, setDrawingMode] = useState(false);
  const [drawPoints, setDrawPoints] = useState<Point[]>([]);
  const [selectedZoneIndex, setSelectedZoneIndex] = useState<number | null>(null);

  const loadFlexZones = async () => {
    try {
      const zones = await invoke<FlexZone[]>("list_flex_zones");
      setFlexZones(zones);
    } catch {}
  };

  useEffect(() => {
    loadFlexZones();
  }, [board]);

  const addZone = async (region?: Point[]) => {
    const zoneRegion = region || board?.outline?.shape?.[0];
    if (!zoneRegion) {
      toast.error("Define a board outline first or draw a region");
      return;
    }
    try {
      await invoke("add_flex_zone", {
        name,
        region: zoneRegion,
        bendRadiusMm: bendRadius,
        numBends,
      });
      toast.success(`Flex zone "${name}" created`);
      setShowForm(false);
      setName("");
      setDrawPoints([]);
      setDrawingMode(false);
      setTool("select");
      await loadFlexZones();
      await loadBoard();
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
  };

  const deleteZone = async (index: number) => {
    try {
      await invoke("delete_flex_zone", { index });
      await loadFlexZones();
      await loadBoard();
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
  };

  const startDrawing = useCallback(() => {
    setDrawingMode(true);
    setDrawPoints([]);
    setTool("outline");
  }, [setTool]);

  const handleCanvasClick = useCallback((pos: Point) => {
    if (!drawingMode) return;
    setDrawPoints(prev => [...prev, pos]);
  }, [drawingMode]);

  const finishDrawing = useCallback(() => {
    if (drawPoints.length >= 3) {
      addZone(drawPoints);
    } else {
      toast.error("Need at least 3 points to define a zone");
    }
  }, [drawPoints, addZone]);

  const cancelDrawing = useCallback(() => {
    setDrawingMode(false);
    setDrawPoints([]);
    setTool("select");
  }, [setTool]);

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <h3 className="font-semibold text-sm">Rigid-Flex Zones</h3>

      {/* Mode selector */}
      <div className="flex gap-1 mb-2">
        <button
          onClick={() => { setShowForm(false); setDrawingMode(false); setTool("select"); }}
          className={`flex-1 px-2 py-1 text-xs rounded ${!showForm && !drawingMode ? "bg-copper text-white" : "bg-bg-elevated text-text-secondary"}`}
        >
          List
        </button>
        <button
          onClick={() => { setShowForm(true); setDrawingMode(false); }}
          className={`flex-1 px-2 py-1 text-xs rounded ${showForm ? "bg-copper text-white" : "bg-bg-elevated text-text-secondary"}`}
        >
          Create (Outline)
        </button>
        <button
          onClick={startDrawing}
          className={`flex-1 px-2 py-1 text-xs rounded ${drawingMode ? "bg-copper text-white" : "bg-bg-elevated text-text-secondary"}`}
        >
          Draw Zone
        </button>
      </div>

      {/* Draw mode active - show instructions */}
      {drawingMode && (
        <div className="bg-amber-900/30 border border-amber-500/50 rounded p-2 text-[11px] text-amber-300 space-y-1">
          <p><strong>Draw Mode Active</strong> - Click to add vertices</p>
          <p>Points: {drawPoints.length} (need ≥3)</p>
          <div className="flex gap-1">
            <button onClick={finishDrawing} className="flex-1 px-2 py-1 bg-green-900/50 border border-green-500/50 rounded text-xs">
              Finish
            </button>
            <button onClick={cancelDrawing} className="flex-1 px-2 py-1 bg-red-900/50 border border-red-500/50 rounded text-xs">
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Create form using board outline */}
      {showForm && !drawingMode && (
        <div className="bg-bg-elevated rounded p-2 space-y-2">
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Zone name"
            className="w-full px-2 py-1 bg-bg-surface border border-border rounded text-xs"
          />
          <div className="flex gap-2 items-center">
            <span className="text-text-muted">Bend radius:</span>
            <input
              type="number"
              value={bendRadius}
              onChange={(e) => setBendRadius(Number(e.target.value))}
              className="w-16 px-1 py-0.5 bg-bg-surface border border-border rounded text-xs"
              step={0.5}
              min={1}
            />
            <span className="text-text-muted">mm</span>
          </div>
          <div className="flex gap-2 items-center">
            <span className="text-text-muted">Bends:</span>
            <input
              type="number"
              value={numBends}
              onChange={(e) => setNumBends(Number(e.target.value))}
              className="w-16 px-1 py-0.5 bg-bg-surface border border-border rounded text-xs"
              min={1}
            />
          </div>
          <div className="flex gap-2">
            <button onClick={() => addZone()} className="flex-1 px-2 py-1 bg-copper text-white rounded text-xs">
              Create from Board Outline
            </button>
            <button onClick={() => setShowForm(false)} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs">
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Zone list with selection */}
      <div className="space-y-1 max-h-64 overflow-y-auto">
        {flexZones.map((z, i) => (
          <div key={i} className={`bg-bg-elevated rounded p-2 space-y-1 ${selectedZoneIndex === i ? "border border-copper/50" : ""}`}>
            <div className="flex justify-between items-center">
              <span className="font-medium">{z.name}</span>
              <div className="flex gap-1">
                <button
                  onClick={() => setSelectedZoneIndex(selectedZoneIndex === i ? null : i)}
                  className="text-copper hover:text-copper/70 text-[11px]"
                >
                  {selectedZoneIndex === i ? "Deselect" : "Select"}
                </button>
                <button
                  onClick={() => deleteZone(i)}
                  className="text-red-500 hover:text-red-400 text-[11px]"
                >
                  Delete
                </button>
              </div>
            </div>
            <div className="flex justify-between text-[11px] text-text-muted">
              <span>Bend radius: {z.bend_radius_mm} mm</span>
              <span>Bends: {z.num_bends}</span>
            </div>
            {selectedZoneIndex === i && z.region.length > 0 && (
              <div className="text-[10px] text-text-muted font-mono">
                {z.region.map((p, idx) => `${idx}: (${p.x.toFixed(1)}, ${p.y.toFixed(1)})`).join(" → ")}
              </div>
            )}
          </div>
        ))}
        {flexZones.length === 0 && (
          <p className="text-text-muted italic">No flex zones defined.</p>
        )}
      </div>
    </div>
  );
}
