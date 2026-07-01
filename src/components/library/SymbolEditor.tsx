import React, { useState, useCallback, useRef, useEffect } from "react";
import { Stage, Layer, Rect, Circle, Line, Group, Text } from "react-konva";
import { KonvaEventObject } from "konva/lib/Node";
import { SymbolDef, PinDef, SymbolGraphics, PinType } from "../../types/library";
import { Point, snap } from "../../lib/geometry";

interface SymbolEditorProps {
  initial?: SymbolDef;
  onSave: (symbol: SymbolDef) => void;
  onClose?: () => void;
  width?: number;
  height?: number;
}

type DrawMode = "select" | "rect" | "circle" | "line" | "text" | "pin";

const GRID = 10;
const PIN_TYPES: PinType[] = ["Input", "Output", "Bidirectional", "Power", "Gnd", "Passive"];

function cloneSymbol(s: SymbolDef): SymbolDef {
  return JSON.parse(JSON.stringify(s));
}

export function SymbolEditor({
  initial,
  onSave,
  onClose,
  width = 800,
  height = 600,
}: SymbolEditorProps) {
  const [symbol, setSymbol] = useState<SymbolDef>(
    initial ? cloneSymbol(initial) : { pins: [], body: [], origin: { x: 0, y: 0 } }
  );
  const [history, setHistory] = useState<SymbolDef[]>([]);
  const [future, setFuture] = useState<SymbolDef[]>([]);
  const [drawMode, setDrawMode] = useState<DrawMode>("select");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [nextPinNumber, setNextPinNumber] = useState(
    initial ? Math.max(0, ...initial.pins.map((p) => parseInt(p.number) || 0)) + 1 : 1
  );

  const [isDrawing, setIsDrawing] = useState(false);
  const [drawStart, setDrawStart] = useState<Point | null>(null);
  const [drawCurrent, setDrawCurrent] = useState<Point | null>(null);
  const [showPinPanel, setShowPinPanel] = useState(false);

  const stageRef = useRef<any>(null);

  const pushHistory = useCallback((s: SymbolDef) => {
    setHistory((h) => [...h.slice(-49), cloneSymbol(s)]);
    setFuture([]);
  }, []);

  const updateSymbol = useCallback(
    (updater: (prev: SymbolDef) => SymbolDef) => {
      setSymbol((prev) => {
        const next = updater(cloneSymbol(prev));
        return next;
      });
      pushHistory(symbol);
    },
    [symbol, pushHistory]
  );

  const undo = useCallback(() => {
    if (history.length === 0) return;
    const prev = history[history.length - 1];
    setHistory((h) => h.slice(0, -1));
    setFuture((f) => [cloneSymbol(symbol), ...f]);
    setSymbol(prev);
    setSelectedId(null);
  }, [history, symbol]);

  const redo = useCallback(() => {
    if (future.length === 0) return;
    const next = future[0];
    setFuture((f) => f.slice(1));
    setHistory((h) => [...h, cloneSymbol(symbol)]);
    setSymbol(next);
    setSelectedId(null);
  }, [future, symbol]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "z") {
        e.preventDefault();
        if (e.shiftKey) redo();
        else undo();
      }
      if (e.key === "Escape") {
        setSelectedId(null);
        setShowPinPanel(false);
        setIsDrawing(false);
        setDrawStart(null);
        setDrawCurrent(null);
      }
      if (e.key === "Delete" || e.key === "Backspace") {
        if (!selectedId) return;
        updateSymbol((prev) => {
          if (selectedId.startsWith("pin-")) {
            const idx = parseInt(selectedId.slice(4));
            return { ...prev, pins: prev.pins.filter((_, i) => i !== idx) };
          }
          if (selectedId.startsWith("body-")) {
            const idx = parseInt(selectedId.slice(5));
            return { ...prev, body: prev.body.filter((_, i) => i !== idx) };
          }
          return prev;
        });
        setSelectedId(null);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedId, undo, redo, updateSymbol]);

  const getPointerPos = (e: KonvaEventObject<MouseEvent | TouchEvent>): Point => {
    const stage = e.target.getStage();
    if (!stage) return { x: 0, y: 0 };
    const pos = stage.getPointerPosition();
    if (!pos) return { x: 0, y: 0 };
    return snap({ x: pos.x, y: pos.y }, GRID);
  };

  const handleMouseDown = (e: KonvaEventObject<MouseEvent>) => {
    if (e.target !== e.target.getStage()) {
      if (drawMode === "select") {
        const targetId = e.target.id() || e.target.findAncestor("Group")?.id();
        if (targetId) {
          setSelectedId(targetId);
          if (targetId.startsWith("pin-")) setShowPinPanel(true);
          else setShowPinPanel(false);
          return;
        }
      }
      return;
    }
    if (drawMode === "select") {
      setSelectedId(null);
      setShowPinPanel(false);
      return;
    }

    const pos = getPointerPos(e);
    setIsDrawing(true);
    setDrawStart(pos);
    setDrawCurrent(pos);
  };

  const handleMouseMove = (e: KonvaEventObject<MouseEvent>) => {
    if (!isDrawing || !drawStart) return;
    const pos = getPointerPos(e);
    setDrawCurrent(pos);
  };

  const handleMouseUp = (e: KonvaEventObject<MouseEvent>) => {
    if (!isDrawing || !drawStart) return;
    const pos = getPointerPos(e);
    const start = drawStart;

    if (drawMode === "pin") {
      if (start.x === pos.x && start.y === pos.y) {
        const pin: PinDef = {
          name: `Pin${nextPinNumber}`,
          number: nextPinNumber.toString(),
          position: start,
          length: GRID * 3,
          electrical_type: "Passive",
        };
        setSymbol((prev) => ({ ...prev, pins: [...prev.pins, pin] }));
        setNextPinNumber((n) => n + 1);
        pushHistory(symbol);
      }
    } else if (drawMode === "text") {
      const textContent = prompt("Enter text:", "Text");
      if (textContent) {
        const g: SymbolGraphics = {
          type: "Text",
          text: textContent,
          position: start,
          size: GRID * 1.2,
        };
        setSymbol((prev) => ({ ...prev, body: [...prev.body, g] }));
        pushHistory(symbol);
      }
    } else if (drawMode === "rect") {
      const g: SymbolGraphics = {
        type: "Rectangle",
        position: {
          x: (start.x + pos.x) / 2,
          y: (start.y + pos.y) / 2,
        },
        size: {
          width: Math.abs(pos.x - start.x),
          height: Math.abs(pos.y - start.y),
        },
        fill: "transparent",
        stroke: "#E8E9F3",
      };
      setSymbol((prev) => ({ ...prev, body: [...prev.body, g] }));
      pushHistory(symbol);
    } else if (drawMode === "circle") {
      const radius = Math.sqrt(
        (pos.x - start.x) ** 2 + (pos.y - start.y) ** 2
      );
      if (radius > 0) {
        const g: SymbolGraphics = {
          type: "Circle",
          center: start,
          radius: snap({ x: radius, y: 0 }, GRID).x || GRID,
          fill: "transparent",
          stroke: "#E8E9F3",
        };
        setSymbol((prev) => ({ ...prev, body: [...prev.body, g] }));
        pushHistory(symbol);
      }
    } else if (drawMode === "line") {
      const g: SymbolGraphics = {
        type: "Line",
        start,
        end: pos,
        stroke: "#E8E9F3",
        width: 1,
      };
      setSymbol((prev) => ({ ...prev, body: [...prev.body, g] }));
      pushHistory(symbol);
    }

    setIsDrawing(false);
    setDrawStart(null);
    setDrawCurrent(null);
  };

  const updatePin = (index: number, updates: Partial<PinDef>) => {
    setSymbol((prev) => {
      const pins = [...prev.pins];
      pins[index] = { ...pins[index], ...updates };
      return { ...prev, pins };
    });
    pushHistory(symbol);
  };

  const renderGrid = () => {
    const lines: React.ReactNode[] = [];
    const w = width;
    const h = height;
    for (let x = 0; x <= w; x += GRID) {
      lines.push(
        <Line
          key={`gv-${x}`}
          points={[x, 0, x, h]}
          stroke="#2A2C45"
          strokeWidth={x % (GRID * 5) === 0 ? 0.6 : 0.3}
        />
      );
    }
    for (let y = 0; y <= h; y += GRID) {
      lines.push(
        <Line
          key={`gh-${y}`}
          points={[0, y, w, y]}
          stroke="#2A2C45"
          strokeWidth={y % (GRID * 5) === 0 ? 0.6 : 0.3}
        />
      );
    }
    return lines;
  };

  const renderBodyGraphics = () => {
    return symbol.body.map((g, i) => {
      const id = `body-${i}`;
      const isSelected = selectedId === id;
      const selColor = "#D47A2F";

      if (g.type === "Rectangle") {
        return (
          <Group key={id} id={id} draggable={drawMode === "select"}
            onDragEnd={(e) => {
              updateSymbol((prev) => {
                const body = [...prev.body];
                const old = body[i] as typeof g;
                body[i] = { ...old, position: { x: e.target.x(), y: e.target.y() } };
                return { ...prev, body };
              });
            }}
            onClick={() => { setSelectedId(id); setShowPinPanel(false); }}
          >
            <Rect
              x={g.position.x - g.size.width / 2}
              y={g.position.y - g.size.height / 2}
              width={g.size.width}
              height={g.size.height}
              fill={g.fill || "transparent"}
              stroke={isSelected ? selColor : (g.stroke || "#E8E9F3")}
              strokeWidth={isSelected ? 1.5 : 0.8}
            />
          </Group>
        );
      }
      if (g.type === "Circle") {
        return (
          <Group key={id} id={id} draggable={drawMode === "select"}
            onDragEnd={(e) => {
              updateSymbol((prev) => {
                const body = [...prev.body];
                const old = body[i] as typeof g;
                body[i] = { ...old, center: { x: e.target.x(), y: e.target.y() } };
                return { ...prev, body };
              });
            }}
            onClick={() => { setSelectedId(id); setShowPinPanel(false); }}
          >
            <Circle
              x={g.center.x}
              y={g.center.y}
              radius={g.radius}
              fill={g.fill || "transparent"}
              stroke={isSelected ? selColor : (g.stroke || "#E8E9F3")}
              strokeWidth={isSelected ? 1.5 : 0.8}
            />
          </Group>
        );
      }
      if (g.type === "Line") {
        return (
          <Group key={id} id={id} draggable={drawMode === "select"}
            onDragEnd={(e) => {
              const dx = e.target.x();
              const dy = e.target.y();
              updateSymbol((prev) => {
                const body = [...prev.body];
                const old = body[i] as typeof g;
                body[i] = {
                  ...old,
                  start: { x: old.start.x + dx, y: old.start.y + dy },
                  end: { x: old.end.x + dx, y: old.end.y + dy },
                };
                return { ...prev, body };
              });
              e.target.position({ x: 0, y: 0 });
            }}
            onClick={() => { setSelectedId(id); setShowPinPanel(false); }}
          >
            <Line
              points={[g.start.x, g.start.y, g.end.x, g.end.y]}
              stroke={isSelected ? selColor : (g.stroke || "#E8E9F3")}
              strokeWidth={isSelected ? 1.5 : (g.width || 0.8)}
            />
          </Group>
        );
      }
      if (g.type === "Text") {
        return (
          <Group key={id} id={id} draggable={drawMode === "select"}
            onDragEnd={(e) => {
              updateSymbol((prev) => {
                const body = [...prev.body];
                const old = body[i] as typeof g;
                body[i] = { ...old, position: { x: e.target.x(), y: e.target.y() } };
                return { ...prev, body };
              });
            }}
            onClick={() => { setSelectedId(id); setShowPinPanel(false); }}
          >
            <Text
              text={g.text}
              x={g.position.x}
              y={g.position.y}
              fontSize={g.size || 12}
              fill={isSelected ? selColor : "#E8E9F3"}
            />
          </Group>
        );
      }
      return null;
    });
  };

  const renderPins = () => {
    return symbol.pins.map((pin, i) => {
      const id = `pin-${i}`;
      const isSelected = selectedId === id;
      const selColor = "#D47A2F";
      const dir = pin.position.x < 0 ? -1 : 1;
      const tip = { x: pin.position.x + dir * pin.length, y: pin.position.y };

      return (
        <Group key={id} id={id} draggable={drawMode === "select"}
          onDragEnd={(e) => {
            const snapped = snap({ x: e.target.x(), y: e.target.y() }, GRID);
            updateSymbol((prev) => {
              const pins = [...prev.pins];
              pins[i] = { ...pins[i], position: snapped };
              return { ...prev, pins };
            });
            e.target.position(snapped);
          }}
          onClick={() => { setSelectedId(id); setShowPinPanel(true); }}
        >
          <Line
            points={[pin.position.x, pin.position.y, tip.x, tip.y]}
            stroke={isSelected ? selColor : "#E8E9F3"}
            strokeWidth={isSelected ? 1.5 : 1}
          />
          <Circle
            x={tip.x}
            y={tip.y}
            radius={3}
            fill={isSelected ? selColor : "#E8E9F3"}
          />
          <Text
            text={pin.name}
            x={tip.x + (dir < 0 ? -pin.length - 2 : 2)}
            y={tip.y - 5}
            fontSize={8}
            fill="#9496B8"
          />
          <Text
            text={pin.number}
            x={pin.position.x + (dir < 0 ? 2 : -pin.length - 2)}
            y={pin.position.y - 5}
            fontSize={7}
            fill="#5C5E7E"
          />
        </Group>
      );
    });
  };

  const renderDrawingPreview = () => {
    if (!isDrawing || !drawStart || !drawCurrent) return null;

    if (drawMode === "rect") {
      const x = Math.min(drawStart.x, drawCurrent.x);
      const y = Math.min(drawStart.y, drawCurrent.y);
      const w = Math.abs(drawCurrent.x - drawStart.x);
      const h = Math.abs(drawCurrent.y - drawStart.y);
      return (
        <Rect
          x={x}
          y={y}
          width={w}
          height={h}
          fill="transparent"
          stroke="#D47A2F"
          strokeWidth={0.8}
          dash={[4, 4]}
        />
      );
    }
    if (drawMode === "circle") {
      const r = Math.sqrt(
        (drawCurrent.x - drawStart.x) ** 2 + (drawCurrent.y - drawStart.y) ** 2
      );
      return (
        <Circle
          x={drawStart.x}
          y={drawStart.y}
          radius={r}
          fill="transparent"
          stroke="#D47A2F"
          strokeWidth={0.8}
          dash={[4, 4]}
        />
      );
    }
    if (drawMode === "line") {
      return (
        <Line
          points={[drawStart.x, drawStart.y, drawCurrent.x, drawCurrent.y]}
          stroke="#D47A2F"
          strokeWidth={0.8}
          dash={[4, 4]}
        />
      );
    }
    if (drawMode === "pin") {
      const dir = drawStart.x < drawCurrent.x ? GRID * 3 : -GRID * 3;
      const tipX = drawStart.x + dir;
      return (
        <Line
          points={[drawStart.x, drawStart.y, tipX, drawStart.y]}
          stroke="#D47A2F"
          strokeWidth={1}
          dash={[4, 4]}
        />
      );
    }
    return null;
  };

  const selectedPin = selectedId?.startsWith("pin-")
    ? symbol.pins[parseInt(selectedId.slice(4))]
    : null;
  const selectedPinIndex = selectedId?.startsWith("pin-")
    ? parseInt(selectedId.slice(4))
    : null;

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-1 p-2 border-b border-border bg-bg-surface">
        {(["select", "rect", "circle", "line", "text"] as DrawMode[]).map((mode) => (
          <button
            key={mode}
            onClick={() => { setDrawMode(mode); setSelectedId(null); setShowPinPanel(false); }}
            className={`px-2 py-1 text-xs rounded ${
              drawMode === mode
                ? "bg-copper text-white"
                : "bg-bg-elevated text-text-secondary hover:text-text-primary"
            }`}
          >
            {mode === "select" ? "Select" : mode === "rect" ? "Rect" : mode === "circle" ? "Circle" : mode === "line" ? "Line" : "Text"}
          </button>
        ))}
        <button
          onClick={() => { setDrawMode("pin"); setSelectedId(null); setShowPinPanel(false); }}
          className={`px-2 py-1 text-xs rounded ${
            drawMode === "pin"
              ? "bg-copper text-white"
              : "bg-bg-elevated text-text-secondary hover:text-text-primary"
          }`}
        >
          Pin
        </button>
        <div className="w-px h-5 bg-border mx-1" />
        <button
          onClick={undo}
          disabled={history.length === 0}
          className="px-2 py-1 text-xs rounded bg-bg-elevated text-text-secondary hover:text-text-primary disabled:opacity-30"
          title="Undo (Ctrl+Z)"
        >
          Undo
        </button>
        <button
          onClick={redo}
          disabled={future.length === 0}
          className="px-2 py-1 text-xs rounded bg-bg-elevated text-text-secondary hover:text-text-primary disabled:opacity-30"
          title="Redo (Ctrl+Shift+Z)"
        >
          Redo
        </button>
        <div className="flex-1" />
        <span className="text-xs text-text-muted mr-2">
          {symbol.pins.length} pins &middot; {symbol.body.length} shapes
        </span>
        {onClose && (
          <button
            onClick={onClose}
            className="px-2 py-1 text-xs rounded bg-bg-elevated text-text-secondary hover:text-text-primary"
          >
            Cancel
          </button>
        )}
        <button
          onClick={() => {
            pushHistory(symbol);
            onSave(symbol);
          }}
          className="px-3 py-1 text-xs rounded bg-success text-white hover:opacity-90"
        >
          Save
        </button>
      </div>
      <div className="flex flex-1 overflow-hidden">
        <div
          className="flex-1 bg-bg-canvas"
          style={{ position: "relative" }}
        >
          <Stage
            ref={stageRef}
            width={width - (showPinPanel && selectedPin ? 200 : 0)}
            height={height - 40}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
          >
            <Layer>
              {renderGrid()}
              {renderBodyGraphics()}
              {renderPins()}
              {renderDrawingPreview()}
            </Layer>
          </Stage>
        </div>
        {showPinPanel && selectedPin && selectedPinIndex !== null && (
          <div className="w-50 border-l border-border bg-bg-surface p-3 space-y-3 overflow-y-auto">
            <h4 className="text-xs font-semibold text-text-primary uppercase tracking-wider">Pin Properties</h4>
            <div className="space-y-2">
              <label className="block text-[10px] text-text-muted">Name</label>
              <input
                type="text"
                value={selectedPin.name}
                onChange={(e) => updatePin(selectedPinIndex, { name: e.target.value })}
                className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-[10px] text-text-muted">Number</label>
              <input
                type="text"
                value={selectedPin.number}
                onChange={(e) => updatePin(selectedPinIndex, { number: e.target.value })}
                className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-[10px] text-text-muted">Length</label>
              <input
                type="number"
                value={selectedPin.length}
                onChange={(e) => updatePin(selectedPinIndex, { length: parseFloat(e.target.value) || 0 })}
                className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-[10px] text-text-muted">Electrical Type</label>
              <select
                value={selectedPin.electrical_type}
                onChange={(e) => updatePin(selectedPinIndex, { electrical_type: e.target.value as PinType })}
                className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
              >
                {PIN_TYPES.map((t) => (
                  <option key={t} value={t}>{t}</option>
                ))}
              </select>
            </div>
            <div className="space-y-2">
              <label className="block text-[10px] text-text-muted">Position X</label>
              <input
                type="number"
                value={selectedPin.position.x}
                onChange={(e) => updatePin(selectedPinIndex, { position: { ...selectedPin.position, x: parseFloat(e.target.value) || 0 } })}
                className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
              />
            </div>
            <div className="space-y-2">
              <label className="block text-[10px] text-text-muted">Position Y</label>
              <input
                type="number"
                value={selectedPin.position.y}
                onChange={(e) => updatePin(selectedPinIndex, { position: { ...selectedPin.position, y: parseFloat(e.target.value) || 0 } })}
                className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
