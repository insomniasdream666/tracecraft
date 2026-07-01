import React, { useEffect, useRef, useCallback, useMemo } from "react";
import { Layer, Line, Circle, Group, Rect, Text } from "react-konva";
import { invoke } from "@tauri-apps/api/core";
import { KonvaCanvas } from "../common/KonvaCanvas";
import { usePCBStore } from "../../stores/pcbStore";
import { useUIStore } from "../../stores/uiStore";
import { Point } from "../../lib/geometry";
import { ZoneRenderer } from "./ZoneRenderer";
import { DRCOverlay } from "./DRCOverlay";
import { FootprintRenderer } from "./FootprintRenderer";
import { TraceRenderer } from "./TraceRenderer";

function isInViewport(
  point: Point,
  vp: { x: number; y: number; width: number; height: number }
) {
  return (
    point.x >= vp.x &&
    point.x <= vp.x + vp.width &&
    point.y >= vp.y &&
    point.y <= vp.y + vp.height
  );
}

function boxInViewport(
  box: { x: number; y: number; width: number; height: number },
  vp: { x: number; y: number; width: number; height: number }
) {
  return (
    box.x < vp.x + vp.width &&
    box.x + box.width > vp.x &&
    box.y < vp.y + vp.height &&
    box.y + box.height > vp.y
  );
}

function anyPointInViewport(
  points: Point[],
  vp: { x: number; y: number; width: number; height: number }
) {
  for (const p of points) {
    if (isInViewport(p, vp)) return true;
  }
  return false;
}

function findNearVertex(pos: Point, vertices: Point[], threshold = 10): number {
  for (let i = vertices.length - 1; i >= 0; i--) {
    const dx = pos.x - vertices[i].x;
    const dy = pos.y - vertices[i].y;
    if (Math.sqrt(dx * dx + dy * dy) < threshold) return i;
  }
  return -1;
}

export const PcbCanvas = React.memo(function PcbCanvas() {
  const { board, loadBoard, selectedIds, ratsnest, activeLayer, visibleLayers, drcViolations, error, footprintCache, select, clearSelection, loadRatsnest } = usePCBStore();
  const { activeTool, panOffset, zoom } = useUIStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = React.useState({ width: 800, height: 600 });
  const [tracePoints, setTracePoints] = React.useState<Point[]>([]);
  const [zonePoints, setZonePoints] = React.useState<Point[]>([]);
  const [selectionRect, setSelectionRect] = React.useState<{ x: number; y: number; w: number; h: number } | null>(null);
  const isDragging = useRef(false);
  const dragStart = useRef<Point | null>(null);
  const [outlineVertices, setOutlineVertices] = React.useState<Point[]>([]);
  const [draggingVertex, setDraggingVertex] = React.useState<number | null>(null);
  const [ghostPos, setGhostPos] = React.useState<Point | null>(null);
  const lastRatsnestCall = useRef(0);
  const ratsnestTimer = useRef<ReturnType<typeof setInterval> | null>(null);

  const viewport = useMemo(() => {
    const left = -panOffset.x / zoom;
    const top = -panOffset.y / zoom;
    const right = (-panOffset.x + size.width) / zoom;
    const bottom = (-panOffset.y + size.height) / zoom;
    const pad = 200;
    return { x: left - pad, y: top - pad, width: right - left + pad * 2, height: bottom - top + pad * 2 };
  }, [panOffset, zoom, size]);

  const throttledLoadRatsnest = useCallback(() => {
    const now = Date.now();
    if (now - lastRatsnestCall.current > 500) {
      lastRatsnestCall.current = now;
      loadRatsnest();
    }
  }, [loadRatsnest]);

  useEffect(() => {
    loadBoard().then(() => {
      throttledLoadRatsnest();
    });
  }, [loadBoard, throttledLoadRatsnest]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setSize({ width: entry.contentRect.width, height: entry.contentRect.height });
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    ratsnestTimer.current = setInterval(() => {
      throttledLoadRatsnest();
    }, 5000);
    return () => {
      if (ratsnestTimer.current) clearInterval(ratsnestTimer.current);
    };
  }, [throttledLoadRatsnest]);

  const handleWorldMouseDown = useCallback(async (pos: Point) => {
    if (activeTool === "component") {
    } else if (activeTool === "route") {
      setTracePoints((prev) => [...prev, pos]);
    } else if (activeTool === "zone") {
      setZonePoints((prev) => [...prev, pos]);
    } else if (activeTool === "outline") {
      const hitIdx = findNearVertex(pos, outlineVertices);
      if (hitIdx >= 0) {
        setDraggingVertex(hitIdx);
      } else {
        setOutlineVertices((prev) => [...prev, pos]);
        setGhostPos(null);
      }
    } else if (activeTool === "select") {
      const hit = findElementAt(pos);
      if (hit) {
        select(hit.id, hit.shiftKey);
      } else {
        isDragging.current = true;
        dragStart.current = pos;
      }
    }
  }, [activeTool, select, outlineVertices]);

  const handleWorldDblClick = useCallback(async (pos: Point) => {
    if (activeTool === "route" && tracePoints.length >= 2) {
      await usePCBStore.getState().routeTrace(tracePoints, activeLayer, 0.25);
      setTracePoints([]);
    } else if (activeTool === "zone" && zonePoints.length >= 3) {
      await usePCBStore.getState().drawZone(zonePoints);
      setZonePoints([]);
    } else if (activeTool === "outline" && outlineVertices.length >= 3) {
      await invoke("set_board_outline", { shape: [outlineVertices], cutouts: [] });
      setOutlineVertices([]);
      await loadBoard();
    } else if (activeTool === "select") {
    }
  }, [activeTool, tracePoints, zonePoints, outlineVertices, activeLayer, loadBoard]);

  const handleWorldMouseMove = useCallback((pos: Point) => {
    if (activeTool === "outline") {
      if (draggingVertex !== null) {
        setOutlineVertices((prev) => {
          const next = [...prev];
          next[draggingVertex] = pos;
          return next;
        });
      } else {
        setGhostPos(pos);
      }
      return;
    }
    if (isDragging.current && dragStart.current) {
      const x = Math.min(dragStart.current.x, pos.x);
      const y = Math.min(dragStart.current.y, pos.y);
      const w = Math.abs(pos.x - dragStart.current.x);
      const h = Math.abs(pos.y - dragStart.current.y);
      setSelectionRect({ x, y, w, h });
    }
  }, [activeTool, draggingVertex]);

  const handleWorldMouseUp = useCallback((pos: Point) => {
    if (draggingVertex !== null) {
      setDraggingVertex(null);
      throttledLoadRatsnest();
      return;
    }
    if (isDragging.current && selectionRect) {
      if (board) {
        for (const [id, fp] of Object.entries(board.footprints)) {
          if (fp.position.x >= selectionRect.x && fp.position.x <= selectionRect.x + selectionRect.w &&
              fp.position.y >= selectionRect.y && fp.position.y <= selectionRect.y + selectionRect.h) {
            select(id, true);
          }
        }
      }
    }
    if (isDragging.current) {
      throttledLoadRatsnest();
    }
    isDragging.current = false;
    dragStart.current = null;
    setSelectionRect(null);
  }, [selectionRect, board, select, draggingVertex, throttledLoadRatsnest]);

  const findElementAt = (pos: Point): { id: string; shiftKey: boolean } | null => {
    if (!board) return null;
    for (const [id, fp] of Object.entries(board.footprints)) {
      if (Math.abs(fp.position.x - pos.x) < 6 && Math.abs(fp.position.y - pos.y) < 6) {
        return { id, shiftKey: false };
      }
    }
    for (const [id, trace] of Object.entries(board.traces)) {
      for (const pt of trace.points) {
        if (Math.abs(pt.x - pos.x) < 3 && Math.abs(pt.y - pos.y) < 3) {
          return { id: `trace-${id}`, shiftKey: false };
        }
      }
    }
    return null;
  };

  const handleKeyDown = useCallback(async (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
      e.preventDefault();
      await usePCBStore.getState().undo();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && (e.key === "y" || (e.key === "z" && e.shiftKey))) {
      e.preventDefault();
      await usePCBStore.getState().redo();
      return;
    }
    if (e.key === "Escape") {
      setTracePoints([]);
      setZonePoints([]);
    }
    if ((e.key === "Delete" || e.key === "Backspace") && selectedIds.size > 0) {
      for (const id of selectedIds) {
        const parts = id.split("-");
        const elementType = parts.length > 1 ? parts[0] : "footprint";
        const elementId = parseInt(parts[parts.length - 1]);
        if (!isNaN(elementId)) {
          await usePCBStore.getState().deleteElement(elementType, elementId);
        }
      }
      clearSelection?.();
      await loadBoard();
    }
    if (e.key === "v" || e.key === "V") {
      if (tracePoints.length > 0) {
        const last = tracePoints[tracePoints.length - 1];
        await usePCBStore.getState().addVia(last.x, last.y);
      }
    }
  }, [selectedIds, tracePoints, loadBoard]);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const layersArray = board?.layers.layers || [];

  const footprintsContent = useMemo(() => {
    if (!board) return null;
    return Object.entries(board.footprints)
      .filter(([_, fp]) => isInViewport(fp.position, viewport))
      .map(([id, fp]) => {
        const cached = footprintCache[fp.component_id];
        const pads = cached?.pads?.map(p => ({
          number: p.number,
          shape: p.shape,
          x: p.position.x,
          y: p.position.y,
          width: p.size.width,
          height: p.size.height,
          layer: p.layer,
        })) || [];
        const hasVisibleLayer = pads.length === 0 || pads.some(p => visibleLayers.has(p.layer));
        if (!hasVisibleLayer) return null;
        return (
          <FootprintRenderer
            key={id}
            footprint={fp}
            pads={pads}
            selected={selectedIds.has(id)}
            onClick={() => select(id)}
          />
        );
      });
  }, [board, viewport, selectedIds, footprintCache, select, visibleLayers]);

  const tracesContent = useMemo(() => {
    if (!board) return null;
    return Object.entries(board.traces)
      .filter(([_, trace]) => visibleLayers.has(trace.layer) && anyPointInViewport(trace.points, viewport))
      .map(([id, trace]) => (
        <TraceRenderer
          key={`trace-${id}`}
          trace={trace}
          layers={layersArray}
          selected={selectedIds.has(`trace-${id}`)}
        />
      ));
  }, [board, viewport, visibleLayers, layersArray, selectedIds]);

  const viasContent = useMemo(() => {
    if (!board) return null;
    return Object.values(board.vias)
      .filter((via) => {
        const layerVisible = visibleLayers.has(via.start_layer) || visibleLayers.has(via.end_layer);
        return layerVisible && isInViewport(via.position, viewport);
      })
      .map((via, i) => (
        <Circle
          key={`via-${i}`}
          x={via.position.x}
          y={via.position.y}
          radius={via.diameter * 2}
          fill="#D47A2F"
          stroke="#A0793F"
          strokeWidth={0.5}
        />
      ));
  }, [board, viewport, visibleLayers]);

  const ratsnestContent = useMemo(() => {
    return ratsnest
      .filter(l => !l.connected && (isInViewport(l.start, viewport) || isInViewport(l.end, viewport)))
      .map((line, i) => (
        <Line
          key={`ratsnest-${i}`}
          points={[line.start.x, line.start.y, line.end.x, line.end.y]}
          stroke="#5096E0"
          strokeWidth={0.5}
          dash={[4, 4]}
          lineCap="round"
        />
      ));
  }, [ratsnest, viewport]);

  const tracePreviewContent = useMemo(() => {
    if (activeTool !== "route" || tracePoints.length === 0) return null;
    const flat = tracePoints.flatMap(p => [p.x, p.y]);
    const layerColor = board?.layers.layers.find(l => l.id === activeLayer)?.color || "#D47A2F";
    return (
      <Line
        points={flat}
        stroke={layerColor}
        strokeWidth={1.5}
        lineCap="round"
        lineJoin="round"
        dash={[4, 4]}
      />
    );
  }, [activeTool, tracePoints, board, activeLayer]);

  const zonePreviewContent = useMemo(() => {
    if (activeTool !== "zone" || zonePoints.length < 2) return null;
    const flat = zonePoints.flatMap(p => [p.x, p.y]);
    const isClosed = zonePoints.length >= 3;
    return (
      <Line
        points={flat}
        fill={isClosed ? "#42BE6520" : undefined}
        stroke="#42BE65"
        strokeWidth={1.5}
        closed={isClosed}
        dash={isClosed ? undefined : [4, 4]}
      />
    );
  }, [activeTool, zonePoints]);

  const selectionRectContent = useMemo(() => {
    if (!selectionRect) return null;
    return (
      <Rect
        x={selectionRect.x}
        y={selectionRect.y}
        width={selectionRect.w}
        height={selectionRect.h}
        fill="#5096E020"
        stroke="#5096E0"
        strokeWidth={1}
        dash={[4, 4]}
      />
    );
  }, [selectionRect]);

  const boardOutlineContent = useMemo(() => {
    if (!board?.outline) return null;
    return board.outline.shape.map((contour, i) => {
      const flat = contour.flatMap(p => [p.x, p.y]);
      return (
        <Line
          key={`outline-${i}`}
          points={flat}
          stroke="#42BE65"
          strokeWidth={2}
          closed
        />
      );
    });
  }, [board]);

  const handleWorldDrop = useCallback(async (pos: Point, componentId: string) => {
    await usePCBStore.getState().placeFootprint(componentId, pos.x, pos.y);
  }, []);

  return (
    <div ref={containerRef} className="w-full h-full relative">
      <div className="absolute top-2 left-2 z-10 flex gap-1">
        <button
          onClick={() => usePCBStore.getState().runDrc()}
          className="px-2 py-1 text-xs bg-bg-surface border border-border rounded text-text hover:bg-bg-elevated"
        >
          Run DRC
        </button>
        {drcViolations.length > 0 && (
          <span className="px-2 py-1 text-xs rounded bg-red-900/50 text-red-300">
            {drcViolations.length} violation{drcViolations.length > 1 ? "s" : ""}
          </span>
        )}
      </div>
      {error && (
        <div className="absolute top-10 left-2 z-20 px-2 py-1 text-xs bg-red-900/80 text-red-200 rounded border border-red-700">
          {error}
        </div>
      )}
      {board && Object.keys(board.footprints).length === 0 && Object.keys(board.traces).length === 0 && (
        <div className="absolute inset-0 z-10 flex items-center justify-center pointer-events-none">
          <div className="text-text-muted text-xs bg-bg-canvas/80 px-4 py-2 rounded border border-border-subtle">
            Drag components from the library to place them on the board
          </div>
        </div>
      )}
      <KonvaCanvas
        width={size.width}
        height={size.height}
        onWorldMouseDown={handleWorldMouseDown}
        onWorldDblClick={handleWorldDblClick}
        onWorldMouseMove={handleWorldMouseMove}
        onWorldMouseUp={handleWorldMouseUp}
        onWorldDrop={handleWorldDrop}
      >
        <Layer>
          {boardOutlineContent}
          {board && <ZoneRenderer zones={board.zones} visibleLayers={visibleLayers} />}
          {ratsnestContent}
          {tracesContent}
          {tracePreviewContent}
          {zonePreviewContent}
          {viasContent}
          {footprintsContent}
          <DRCOverlay violations={drcViolations} />
          {selectionRectContent}
        </Layer>
      </KonvaCanvas>
    </div>
  );
});
