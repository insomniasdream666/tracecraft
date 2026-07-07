import React, { useEffect, useRef, useCallback, useMemo } from "react";
import { Layer, Circle, Line, Text, Group, Rect } from "react-konva";
import { CircuitBoard } from "lucide-react";
import { KonvaCanvas } from "../common/KonvaCanvas";
import { useSchematicStore } from "../../stores/schematicStore";
import { useUIStore } from "../../stores/uiStore";
import { Point } from "../../lib/geometry";

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

export const SchematicCanvas = React.memo(function SchematicCanvas() {
  const { sheet, loadSheet, selectedIds, select, clearSelection } = useSchematicStore();
  const { activeTool, panOffset, zoom } = useUIStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = React.useState({ width: 800, height: 600 });
  const [wirePoints, setWirePoints] = React.useState<Point[]>([]);
  const stageRef = useRef<any>(null);
  const [ghostPos, setGhostPos] = React.useState<Point | null>(null);
  const [ercViolations, setErcViolations] = React.useState<any[]>([]);

  const viewport = useMemo(() => {
    const left = -panOffset.x / zoom;
    const top = -panOffset.y / zoom;
    const right = (-panOffset.x + size.width) / zoom;
    const bottom = (-panOffset.y + size.height) / zoom;
    const pad = 200;
    return { x: left - pad, y: top - pad, width: right - left + pad * 2, height: bottom - top + pad * 2 };
  }, [panOffset, zoom, size]);

  useEffect(() => {
    loadSheet();
  }, [loadSheet]);

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
    if (stageRef.current) {
      window.__tracecraft_schematic_svg = stageRef.current.toSVG();
      stageRef.current.toDataURL({ mimeType: "image/png", pixelRatio: 2 }).then((url: string) => {
        window.__tracecraft_schematic_png = url;
      });
    }
  }, [sheet]);

  const handleWorldClick = useCallback(async (pos: Point) => {
    if (activeTool === "component") {
      await useSchematicStore.getState().placeSymbol("00000000-0000-0000-0000-000000000000", pos.x, pos.y);
    } else if (activeTool === "wire") {
      setWirePoints((prev) => [...prev, pos]);
    } else if (activeTool === "label") {
      // Prompt for net label name
      const name = prompt("Net label name:");
      if (name && sheet) {
        try {
          const { invoke } = await import("@tauri-apps/api/core");
          await invoke("add_net_label", { sheetId: sheet.id, name, position: pos });
          await loadSheet();
        } catch (e) {
          console.error(e);
        }
      }
    } else if (activeTool === "junction") {
      if (sheet) {
        try {
          const { invoke } = await import("@tauri-apps/api/core");
          await invoke("add_junction", { sheetId: sheet.id, position: pos });
          await loadSheet();
        } catch (e) {
          console.error(e);
        }
      }
    } else if (activeTool === "select") {
      const el = findElementAt(pos);
      if (el) {
        select(el.id, el.shiftKey);
      } else {
        clearSelection();
      }
    }
  }, [activeTool, sheet, loadSheet, select, clearSelection]);

  const handleWorldDblClick = useCallback(async (pos: Point) => {
    if (activeTool === "wire" && wirePoints.length >= 2) {
      await useSchematicStore.getState().drawWire(wirePoints);
      setWirePoints([]);
      setGhostPos(null);
    }
  }, [activeTool, wirePoints]);

  const handleWorldMouseMove = useCallback((pos: Point) => {
    if (activeTool === "wire") {
      setGhostPos(pos);
    }
  }, [activeTool]);

  const findElementAt = (pos: Point): { id: string; shiftKey: boolean } | null => {
    if (!sheet) return null;
    // Check symbols first (top layer)
    for (const [id, sym] of Object.entries(sheet.symbols)) {
      if (Math.abs(sym.position.x - pos.x) < 10 && Math.abs(sym.position.y - pos.y) < 10) {
        return { id, shiftKey: false };
      }
    }
    // Check wires
    for (const [id, wire] of Object.entries(sheet.wires)) {
      for (const pt of wire.points) {
        if (Math.abs(pt.x - pos.x) < 5 && Math.abs(pt.y - pos.y) < 5) {
          return { id: `wire-${id}`, shiftKey: false };
        }
      }
    }
    return null;
  };

  const symbolsContent = useMemo(() => {
    if (!sheet) return null;
    return Object.entries(sheet.symbols)
      .filter(([_, sym]) => isInViewport(sym.position, viewport))
      .map(([id, sym]) => (
        <Group key={id} x={sym.position.x} y={sym.position.y}>
          <Rect
            width={20} height={14} x={-10} y={-7}
            fill="#2D2F52"
            stroke={selectedIds.has(id) ? "#5096E0" : "#5C5E7E"}
            strokeWidth={selectedIds.has(id) ? 2 : 1}
            cornerRadius={2}
          />
          <Text text={sym.refdes || "?"} x={-8} y={-4} fontSize={8} fill="#E8E9F3" />
          {sym.value && (
            <Text text={sym.value} x={-8} y={4} fontSize={6} fill="#8A8AB0" />
          )}
        </Group>
      ));
  }, [sheet, selectedIds, viewport]);

  const wiresContent = useMemo(() => {
    if (!sheet) return null;
    return Object.values(sheet.wires)
      .filter((wire) => wire.points.some((p) => isInViewport(p, viewport)))
      .map((wire, i) => {
        const flat = wire.points.flatMap((p) => [p.x, p.y]);
        return (
          <Line
            key={`wire-${i}`}
            points={flat}
            stroke="#D4783C"
            strokeWidth={wire.width * 2}
            lineCap="round"
            lineJoin="round"
          />
        );
      });
  }, [sheet, viewport]);

  const wirePreviewContent = useMemo(() => {
    if (activeTool !== "wire" || wirePoints.length === 0) return null;
    const flat = wirePoints.flatMap((p) => [p.x, p.y]);
    return (
      <>
        <Line points={flat} stroke="#D4783C" strokeWidth={2} lineCap="round" lineJoin="round" />
        {wirePoints.map((p, i) => (
          <Circle key={i} x={p.x} y={p.y} radius={2} fill="#D4783C" />
        ))}
        {ghostPos && (
          <Line
            points={[wirePoints[wirePoints.length - 1].x, wirePoints[wirePoints.length - 1].y, ghostPos.x, ghostPos.y]}
            stroke="#D4783C60"
            strokeWidth={1}
            dash={[4, 4]}
          />
        )}
      </>
    );
  }, [activeTool, wirePoints, ghostPos]);

  const netLabelsContent = useMemo(() => {
    if (!sheet) return null;
    return Object.entries(sheet.net_labels)
      .filter(([_, label]) => isInViewport(label.position, viewport))
      .map(([id, label]) => (
        <Group key={`label-${id}`} x={label.position.x} y={label.position.y}>
          <Rect
            x={-label.name.length * 3 - 2} y={-7}
            width={label.name.length * 6 + 4} height={14}
            fill="#1A1A2E"
            stroke="#D47A2F"
            strokeWidth={1}
            cornerRadius={2}
          />
          <Text
            text={label.name}
            x={-label.name.length * 3}
            y={-4}
            fontSize={9}
            fill="#D47A2F"
          />
        </Group>
      ));
  }, [sheet, viewport]);

  const junctionsContent = useMemo(() => {
    if (!sheet) return null;
    return Object.values(sheet.junctions)
      .filter((junc) => isInViewport(junc.position, viewport))
      .map((junc, i) => (
        <Circle
          key={`junc-${i}`}
          x={junc.position.x}
          y={junc.position.y}
          radius={3}
          fill="#D4783C"
        />
      ));
  }, [sheet, viewport]);

  const busesContent = useMemo(() => {
    if (!sheet) return null;
    return Object.entries(sheet.buses).map(([id, bus]) => (
      <Group key={`bus-${id}`}>
        <Text
          text={`BUS: ${bus.name}`}
          x={0} y={0}
          fontSize={10}
          fill="#42BE65"
          fontStyle="bold"
        />
        {bus.members.map((m, i) => (
          <Text
            key={i}
            text={`  ${m}`}
            x={0} y={12 + i * 12}
            fontSize={8}
            fill="#42BE65"
          />
        ))}
      </Group>
    ));
  }, [sheet]);

  const ercContent = useMemo(() => {
    return ercViolations
      .filter((v: any) => isInViewport(v.location, viewport))
      .map((v: any, i: number) => (
        <Circle
          key={`erc-${i}`}
          x={v.location.x}
          y={v.location.y}
          radius={5}
          fill={v.severity === "Error" ? "#FF4444" : "#FFAA00"}
          opacity={0.8}
        />
      ));
  }, [ercViolations, viewport]);

  const runERC = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const violations = await invoke<any[]>("run_erc_command");
      setErcViolations(violations);
    } catch (e) {
      console.error(e);
    }
  };

  const handleKeyDown = useCallback(async (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
      e.preventDefault();
      await useSchematicStore.getState().undo();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && (e.key === "y" || (e.key === "z" && e.shiftKey))) {
      e.preventDefault();
      await useSchematicStore.getState().redo();
      return;
    }
    if (e.key === "Escape" && activeTool === "wire") {
      setWirePoints([]);
      setGhostPos(null);
    }
    if (e.key === "Delete" || e.key === "Backspace") {
      if (selectedIds.size > 0 && sheet) {
        const { invoke } = await import("@tauri-apps/api/core");
        for (const id of selectedIds) {
          const parts = id.split("-");
          const elementType = parts.length > 1 ? parts[0] : "symbol";
          const elementId = parseInt(parts[parts.length - 1]);
          if (!isNaN(elementId)) {
            await invoke("delete_element", { sheetId: sheet.id, elementType, elementId });
          }
        }
        clearSelection();
        await loadSheet();
      }
    }
  }, [activeTool, selectedIds, sheet, loadSheet, clearSelection]);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const isEmpty = sheet && Object.keys(sheet.symbols).length === 0 && Object.keys(sheet.wires).length === 0;

  return (
    <div ref={containerRef} className="w-full h-full relative">
      {isEmpty && (
        <div className="absolute inset-0 z-20 flex items-center justify-center pointer-events-none">
          <div className="text-center pointer-events-auto">
            <div className="w-12 h-12 mx-auto mb-3 rounded-xl bg-bg-surface border border-border flex items-center justify-center">
              <CircuitBoard size={24} className="text-copper/60" />
            </div>
            <p className="text-sm text-text-muted">Empty schematic sheet</p>
            <p className="text-xs text-text-muted/60 mt-1">Drag components from the Library tab to begin</p>
          </div>
        </div>
      )}
      <div className="absolute top-2 left-2 z-10 flex gap-1">
        <button
          onClick={runERC}
          className="px-2 py-1 text-xs bg-bg-surface border border-border rounded text-text hover:bg-bg-elevated"
        >
          Run ERC
        </button>
        {ercViolations.length > 0 && (
          <span className="px-2 py-1 text-xs rounded bg-red-900/50 text-red-300">
            {ercViolations.length} violation{ercViolations.length > 1 ? "s" : ""}
          </span>
        )}
      </div>
      <KonvaCanvas
        width={size.width}
        height={size.height}
        onWorldMouseDown={handleWorldClick}
        onWorldDblClick={handleWorldDblClick}
        onWorldMouseMove={handleWorldMouseMove}
        onStageReady={(stage) => {
          stageRef.current = stage;
          window.__tracecraft_schematic_svg = stage.toSVG();
          stage.toDataURL({ mimeType: "image/png", pixelRatio: 2 }).then((url: string) => {
            window.__tracecraft_schematic_png = url;
          });
        }}
      >
        <Layer>
          {wiresContent}
          {wirePreviewContent}
          {junctionsContent}
          {netLabelsContent}
          {busesContent}
          {symbolsContent}
          {ercContent}
        </Layer>
      </KonvaCanvas>
    </div>
  );
});
