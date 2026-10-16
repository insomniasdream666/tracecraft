import React, { useState, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { usePCBStore } from "../../stores/pcbStore";
import toast from "react-hot-toast";
import { Point } from "../../lib/geometry";

export function BusRoutePanel() {
  const { board, loadBoard, activeLayer, selectedIds } = usePCBStore();
  const [netIds, setNetIds] = useState<string>("");
  const [width, setWidth] = useState(0.25);
  const [spacing, setSpacing] = useState(0.5);
  const [routing, setRouting] = useState(false);
  const [startMode, setStartMode] = useState<"click" | "auto">("click");
  const [startPoints, setStartPoints] = useState<Point[]>([]);
  const [endPoints, setEndPoints] = useState<Point[]>([]);
  const [collectingStarts, setCollectingStarts] = useState(false);
  const [collectingEnds, setCollectingEnds] = useState(false);

  const ids = useMemo(() => netIds.split(",").map(s => parseInt(s.trim())).filter(n => !isNaN(n)), [netIds]);

  const selectedNets = useMemo(() => board?.netlist.nets
    .filter((n: any) => !netIds || n.name.includes(netIds))
    .map((n: any) => ({ id: n.id, name: n.name })) || [], [board, netIds]);

  const routeBus = useCallback(async () => {
    if (!board || netIds.trim() === "") return;
    const ids = netIds.split(",").map(s => parseInt(s.trim())).filter(n => !isNaN(n));
    if (ids.length === 0) return toast.error("Enter valid net IDs");

    if (startMode === "click") {
      if (startPoints.length !== ids.length) return toast.error(`Click ${ids.length} start points`);
      if (endPoints.length !== ids.length) return toast.error(`Click ${ids.length} end points`);
    }

    setRouting(true);
    try {
      await invoke("route_bus", {
        netIds: ids,
        layer: activeLayer,
        startPoints: startMode === "auto" ? ids.map(() => ({ x: 0, y: 0 })) : startPoints,
        endPoints: startMode === "auto" ? ids.map(() => ({ x: 10, y: 0 })) : endPoints,
        width,
        spacing,
      });
      toast.success(`Routed ${ids.length} nets as bus`);
      await loadBoard();
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
    setRouting(false);
    setCollectingStarts(false);
    setCollectingEnds(false);
  }, [netIds, startMode, startPoints, endPoints, width, spacing, activeLayer, board, loadBoard]);

  const handleCanvasClick = useCallback((pos: Point) => {
    if (collectingStarts && startPoints.length < ids.length) {
      setStartPoints([...startPoints, pos]);
      if (startPoints.length + 1 === ids.length) {
        setCollectingStarts(false);
        setCollectingEnds(true);
      }
    } else if (collectingEnds && endPoints.length < ids.length) {
      setEndPoints([...endPoints, pos]);
      if (endPoints.length + 1 === ids.length) {
        setCollectingEnds(false);
      }
    }
  }, [collectingStarts, collectingEnds, startPoints, endPoints, ids]);

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <h3 className="font-semibold text-sm">Bus Router</h3>

      <div className="bg-bg-elevated rounded p-2 space-y-2">
        <label className="flex flex-col gap-1 text-[11px]">
          <span className="text-text-muted">Net IDs (comma separated)</span>
          <input
            value={netIds}
            onChange={(e) => setNetIds(e.target.value)}
            placeholder="1,2,3,4"
            className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
          />
        </label>

        <div className="flex gap-2">
          <label className="flex items-center gap-1 text-[11px]">
            <input
              type="radio"
              checked={startMode === "auto"}
              onChange={() => setStartMode("auto")}
            />
            Auto (parallel from center)
          </label>
          <label className="flex items-center gap-1 text-[11px]">
            <input
              type="radio"
              checked={startMode === "click"}
              onChange={() => setStartMode("click")}
            />
            Click Points
          </label>
        </div>

        {startMode === "click" && (
          <div className="space-y-1">
            <button
              onClick={() => { setStartPoints([]); setEndPoints([]); setCollectingStarts(true); }}
              disabled={collectingStarts || collectingEnds}
              className="w-full px-2 py-1 bg-copper/20 border border-copper/30 rounded text-xs hover:bg-copper/30"
            >
              {collectingStarts
                ? `Click start point ${startPoints.length + 1} of ${ids.length}`
                : collectingEnds
                ? `Click end point ${endPoints.length + 1} of ${ids.length}`
                : "Start Point Selection"}
            </button>
            {startPoints.length > 0 && <p className="text-[10px] text-text-muted">Starts: {startPoints.map(p => `(${p.x.toFixed(1)},${p.y.toFixed(1)})`).join(", ")}</p>}
            {endPoints.length > 0 && <p className="text-[10px] text-text-muted">Ends: {endPoints.map(p => `(${p.x.toFixed(1)},${p.y.toFixed(1)})`).join(", ")}</p>}
          </div>
        )}

        <label className="flex flex-col gap-1 text-[11px]">
          <span className="text-text-muted">Trace Width (mm)</span>
          <input
            type="number"
            value={width}
            onChange={(e) => setWidth(Math.max(0.1, Number(e.target.value)))}
            className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
            step={0.05}
          />
        </label>

        <label className="flex flex-col gap-1 text-[11px]">
          <span className="text-text-muted">Spacing (mm)</span>
          <input
            type="number"
            value={spacing}
            onChange={(e) => setSpacing(Math.max(0.1, Number(e.target.value)))}
            className="px-2 py-1 bg-bg-surface border border-border rounded text-xs"
            step={0.05}
          />
        </label>

        <button
          onClick={routeBus}
          disabled={routing || netIds.trim() === "" || (startMode === "click" && (collectingStarts || collectingEnds))}
          className="w-full px-3 py-1.5 bg-copper text-white rounded hover:opacity-90 disabled:opacity-50 text-xs font-medium"
        >
          {routing ? "Routing..." : "Route Bus"}
        </button>
      </div>

      <div className="bg-bg-surface rounded p-2 text-[11px] text-text-muted space-y-1 max-h-32 overflow-y-auto">
        <p className="font-medium">Available Nets:</p>
        {selectedNets.length === 0 ? (
          <p>No nets found</p>
        ) : (
          selectedNets.slice(0, 10).map((n) => (
            <div key={n.id} className="flex justify-between">
              <span>{n.name}</span>
              <span className="font-mono">{n.id}</span>
            </div>
          ))
        )}
        {selectedNets.length > 10 && <p className="text-text-muted">+{selectedNets.length - 10} more...</p>}
      </div>
    </div>
  );
}