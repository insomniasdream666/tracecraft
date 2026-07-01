import React, { useState, useCallback, useRef, useEffect } from "react";
import { Stage, Layer, Rect, Circle, Line, Group, Text, Ellipse } from "react-konva";
import { KonvaEventObject } from "konva/lib/Node";
import { FootprintDef, PadDef, PadShape } from "../../types/library";
import { Point, snap } from "../../lib/geometry";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

interface FootprintEditorProps {
  componentId: string;
  initial?: FootprintDef;
  onSave: (footprint: FootprintDef) => void;
  onClose?: () => void;
  width?: number;
  height?: number;
}

type EditTool = "select" | "pad-round" | "pad-rect" | "pad-oval" | "draw-outline" | "draw-courtyard";

const GRID = 2.54;
const ZOOM = 20;
const HANDLE_RADIUS = 4;

const PAD_SHAPES: { label: string; value: EditTool }[] = [
  { label: "Round", value: "pad-round" },
  { label: "Rect", value: "pad-rect" },
  { label: "Oval", value: "pad-oval" },
];

const LAYER_NAMES: Record<number, string> = {
  1: "Top Copper",
  2: "Bottom Copper",
  31: "Silkscreen Top",
  32: "Silkscreen Bottom",
};

const LAYER_COLORS: Record<number, string> = {
  1: "#D47A2F",
  2: "#4A90D9",
  31: "#E8E9F3",
  32: "#9496B8",
};

function cloneFootprint(fp: FootprintDef): FootprintDef {
  return JSON.parse(JSON.stringify(fp));
}

function toKonva(p: Point): Point {
  return { x: p.x * ZOOM, y: -p.y * ZOOM };
}

function fromKonva(p: Point, doSnap = true): Point {
  const raw = { x: p.x / ZOOM, y: -p.y / ZOOM };
  return doSnap ? snap(raw, GRID) : raw;
}

export function FootprintEditor({
  componentId: _componentId,
  initial,
  onSave,
  onClose,
  width = 800,
  height = 600,
}: FootprintEditorProps) {
  const [footprint, setFootprint] = useState<FootprintDef>(
    initial ? cloneFootprint(initial) : { pads: [], outline: [], courtyard: [] }
  );
  const [tool, setTool] = useState<EditTool>("select");
  const [selectedPadIndex, setSelectedPadIndex] = useState<number | null>(null);
  const [selectedOutlineIndex, setSelectedOutlineIndex] = useState<number | null>(null);
  const [selectedCourtyardIndex, setSelectedCourtyardIndex] = useState<number | null>(null);
  const [drawingPoints, setDrawingPoints] = useState<Point[]>([]);
  const [modelPath, setModelPath] = useState<string | null>(null);
  const [nextPadNumber, setNextPadNumber] = useState(
    initial ? Math.max(0, ...initial.pads.map((p) => parseInt(p.number) || 0)) + 1 : 1
  );

  const stageRef = useRef<any>(null);

  const updateFootprint = useCallback(
    (updater: (prev: FootprintDef) => FootprintDef) => {
      setFootprint((prev) => updater(cloneFootprint(prev)));
    },
    []
  );

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setSelectedPadIndex(null);
        setSelectedOutlineIndex(null);
        setSelectedCourtyardIndex(null);
        setDrawingPoints([]);
      }
      if ((e.key === "Delete" || e.key === "Backspace") && selectedPadIndex !== null) {
        updateFootprint((prev) => ({
          ...prev,
          pads: prev.pads.filter((_, i) => i !== selectedPadIndex),
        }));
        setSelectedPadIndex(null);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedPadIndex, updateFootprint]);

  const getPointerPos = (e: KonvaEventObject<MouseEvent | TouchEvent>): Point => {
    const stage = e.target.getStage();
    if (!stage) return { x: 0, y: 0 };
    const pos = stage.getPointerPosition();
    if (!pos) return { x: 0, y: 0 };
    return fromKonva({ x: pos.x, y: pos.y });
  };

  const handleMouseDown = (e: KonvaEventObject<MouseEvent>) => {
    if (e.target !== e.target.getStage()) {
      if (tool === "select") {
        const targetId = e.target.id() || e.target.findAncestor("Group")?.id();
        if (targetId) {
          if (targetId.startsWith("pad-")) {
            setSelectedPadIndex(parseInt(targetId.slice(4)));
            setSelectedOutlineIndex(null);
            setSelectedCourtyardIndex(null);
          } else if (targetId === "outline-point" || targetId.startsWith("outline-")) {
            const idx = parseInt(targetId.split("-").pop() || "0");
            setSelectedOutlineIndex(idx);
            setSelectedPadIndex(null);
            setSelectedCourtyardIndex(null);
          } else if (targetId === "courtyard-point" || targetId.startsWith("courtyard-")) {
            const idx = parseInt(targetId.split("-").pop() || "0");
            setSelectedCourtyardIndex(idx);
            setSelectedPadIndex(null);
            setSelectedOutlineIndex(null);
          }
          return;
        }
      }
      return;
    }

    setSelectedPadIndex(null);
    setSelectedOutlineIndex(null);
    setSelectedCourtyardIndex(null);

    const pos = getPointerPos(e);

    if (tool === "pad-round" || tool === "pad-rect" || tool === "pad-oval") {
      const shapeMap: Record<string, PadShape> = {
        "pad-round": "Round",
        "pad-rect": "Rect",
        "pad-oval": "Oval",
      };
      const pad: PadDef = {
        number: nextPadNumber.toString(),
        shape: shapeMap[tool],
        position: pos,
        size: { width: GRID * 2, height: GRID * 2 },
        layer: 1,
        drill: tool === "pad-round" ? GRID * 0.6 : null,
      };
      updateFootprint((prev) => ({ ...prev, pads: [...prev.pads, pad] }));
      setNextPadNumber((n) => n + 1);
    }

    if (tool === "draw-outline") {
      setDrawingPoints((prev) => [...prev, pos]);
    }

    if (tool === "draw-courtyard") {
      setDrawingPoints((prev) => [...prev, pos]);
    }
  };

  const handleMouseUp = (_e: KonvaEventObject<MouseEvent>) => {};

  const handleDblClick = (_e: KonvaEventObject<MouseEvent>) => {
    if ((tool === "draw-outline" || tool === "draw-courtyard") && drawingPoints.length >= 2) {
      if (tool === "draw-outline") {
        updateFootprint((prev) => ({
          ...prev,
          outline: drawingPoints,
        }));
      } else {
        updateFootprint((prev) => ({
          ...prev,
          courtyard: drawingPoints,
        }));
      }
      setDrawingPoints([]);
    }
  };

  const handleDragEnd = (e: KonvaEventObject<DragEvent>) => {
    if (selectedPadIndex === null) return;
    const pos = fromKonva({ x: e.target.x(), y: e.target.y() });
    updateFootprint((prev) => {
      const pads = [...prev.pads];
      pads[selectedPadIndex] = { ...pads[selectedPadIndex], position: pos };
      return { ...prev, pads };
    });
  };

  const updatePad = (index: number, updates: Partial<PadDef>) => {
    updateFootprint((prev) => {
      const pads = [...prev.pads];
      pads[index] = { ...pads[index], ...updates };
      return { ...prev, pads };
    });
  };

  const handleModelPick = async () => {
    try {
      const file = await openDialog({
        multiple: false,
        filters: [{ name: "3D Models", extensions: ["step", "stp", "wrl"] }],
      });
      if (file) setModelPath(file);
    } catch (e) {
      console.error("Failed to pick 3D model:", e);
    }
  };

  const renderGrid = () => {
    const lines: React.ReactNode[] = [];
    const w = width / ZOOM;
    const h = height / ZOOM;
    const halfW = w / 2;
    const halfH = h / 2;

    for (let x = -halfW; x <= halfW; x += GRID) {
      const kx = (x + halfW) * ZOOM;
      lines.push(
        <Line
          key={`gv-${x}`}
          points={[kx, 0, kx, height]}
          stroke="#2A2C45"
          strokeWidth={x % (GRID * 5) === 0 ? 0.8 : 0.4}
        />
      );
    }
    for (let y = -halfH; y <= halfH; y += GRID) {
      const ky = (-y + halfH) * ZOOM;
      lines.push(
        <Line
          key={`gh-${y}`}
          points={[0, ky, width, ky]}
          stroke="#2A2C45"
          strokeWidth={y % (GRID * 5) === 0 ? 0.8 : 0.4}
        />
      );
    }
    return lines;
  };

  const renderPads = () => {
    return footprint.pads.map((pad, i) => {
      const id = `pad-${i}`;
      const isSelected = selectedPadIndex === i;
      const color = isSelected ? "#FFD700" : LAYER_COLORS[pad.layer] || "#E8E9F3";
      const kp = toKonva(pad.position);
      const kw = pad.size.width * ZOOM / 2;
      const kh = pad.size.height * ZOOM / 2;

      const renderShape = () => {
        switch (pad.shape) {
          case "Round":
            return (
              <Circle
                x={kp.x}
                y={kp.y}
                radius={Math.max(kw, kh)}
                fill={color}
                stroke={isSelected ? "#FFD700" : "#343760"}
                strokeWidth={isSelected ? 1.5 : 0.6}
                opacity={0.85}
              />
            );
          case "Rect":
            return (
              <Rect
                x={kp.x - kw}
                y={kp.y - kh}
                width={kw * 2}
                height={kh * 2}
                fill={color}
                stroke={isSelected ? "#FFD700" : "#343760"}
                strokeWidth={isSelected ? 1.5 : 0.6}
                opacity={0.85}
              />
            );
          case "Oval":
            return (
              <Ellipse
                x={kp.x}
                y={kp.y}
                radiusX={kw}
                radiusY={kh}
                fill={color}
                stroke={isSelected ? "#FFD700" : "#343760"}
                strokeWidth={isSelected ? 1.5 : 0.6}
                opacity={0.85}
              />
            );
          default:
            return (
              <Rect
                x={kp.x - kw}
                y={kp.y - kh}
                width={kw * 2}
                height={kh * 2}
                fill={color}
                stroke={isSelected ? "#FFD700" : "#343760"}
                strokeWidth={isSelected ? 1.5 : 0.6}
                opacity={0.85}
              />
            );
        }
      };

      return (
        <Group key={id} id={id} draggable={tool === "select"} onDragEnd={handleDragEnd}>
          {renderShape()}
          {pad.drill != null && (
            <Circle
              x={kp.x}
              y={kp.y}
              r={(pad.drill * ZOOM) / 2}
              fill="#0F1020"
            />
          )}
          <Text
            x={kp.x - 8}
            y={kp.y + Math.max(kh, 8) + 2}
            width={16}
            text={pad.number}
            fontSize={7}
            fill="#E8E9F3"
            align="center"
          />
        </Group>
      );
    });
  };

  const renderOutline = () => {
    if (footprint.outline.length < 2) return null;
    const points = footprint.outline.flatMap((p) => {
      const kp = toKonva(p);
      return [kp.x, kp.y];
    });

    return (
      <>
        <Line
          points={points}
          stroke="#42BE65"
          strokeWidth={1.2}
          closed={true}
          fill="transparent"
        />
        {footprint.outline.map((p, i) => {
          const kp = toKonva(p);
          const isSelected = selectedOutlineIndex === i;
          return (
            <Circle
              key={`outline-${i}`}
              id={`outline-${i}`}
              x={kp.x}
              y={kp.y}
              radius={HANDLE_RADIUS}
              fill={isSelected ? "#FFD700" : "#42BE65"}
              stroke="#0F1020"
              strokeWidth={1}
              draggable={tool === "select"}
              onDragEnd={(e) => {
                const newPos = fromKonva({ x: e.target.x(), y: e.target.y() });
                updateFootprint((prev) => {
                  const outline = [...prev.outline];
                  outline[i] = newPos;
                  return { ...prev, outline };
                });
              }}
              onClick={() => {
                if (tool === "select") {
                  setSelectedOutlineIndex(i);
                  setSelectedPadIndex(null);
                  setSelectedCourtyardIndex(null);
                }
              }}
            />
          );
        })}
      </>
    );
  };

  const renderCourtyard = () => {
    if (footprint.courtyard.length < 2) return null;
    const points = footprint.courtyard.flatMap((p) => {
      const kp = toKonva(p);
      return [kp.x, kp.y];
    });

    return (
      <>
        <Line
          points={points}
          stroke="#5C5E7E"
          strokeWidth={0.8}
          dash={[4, 4]}
          closed={true}
          fill="transparent"
        />
        {footprint.courtyard.map((p, i) => {
          const kp = toKonva(p);
          const isSelected = selectedCourtyardIndex === i;
          return (
            <Circle
              key={`courtyard-${i}`}
              id={`courtyard-${i}`}
              x={kp.x}
              y={kp.y}
              radius={HANDLE_RADIUS}
              fill={isSelected ? "#FFD700" : "#5C5E7E"}
              stroke="#0F1020"
              strokeWidth={1}
              draggable={tool === "select"}
              onDragEnd={(e) => {
                const newPos = fromKonva({ x: e.target.x(), y: e.target.y() });
                updateFootprint((prev) => {
                  const courtyard = [...prev.courtyard];
                  courtyard[i] = newPos;
                  return { ...prev, courtyard };
                });
              }}
              onClick={() => {
                if (tool === "select") {
                  setSelectedCourtyardIndex(i);
                  setSelectedPadIndex(null);
                  setSelectedOutlineIndex(null);
                }
              }}
            />
          );
        })}
      </>
    );
  };

  const renderDrawingPreview = () => {
    if (drawingPoints.length === 0) return null;
    const points = drawingPoints.flatMap((p) => {
      const kp = toKonva(p);
      return [kp.x, kp.y];
    });

    const color = tool === "draw-outline" ? "#42BE65" : "#5C5E7E";
    const dash = tool === "draw-courtyard" ? [4, 4] : undefined;

    return (
      <>
        {drawingPoints.map((p, i) => {
          const kp = toKonva(p);
          return (
            <Circle
              key={`draw-${i}`}
              x={kp.x}
              y={kp.y}
              radius={HANDLE_RADIUS}
              fill={color}
              stroke="#0F1020"
              strokeWidth={1}
            />
          );
        })}
        <Line
          points={points}
          stroke={color}
          strokeWidth={1.2}
          dash={dash}
        />
      </>
    );
  };

  const selectedPad = selectedPadIndex !== null ? footprint.pads[selectedPadIndex] : null;

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-1 p-2 border-b border-border bg-bg-surface">
        {(["select"] as EditTool[]).map((t) => (
          <button
            key={t}
            onClick={() => { setTool(t); setDrawingPoints([]); setSelectedPadIndex(null); setSelectedOutlineIndex(null); setSelectedCourtyardIndex(null); }}
            className={`px-2 py-1 text-xs rounded ${
              tool === t ? "bg-copper text-white" : "bg-bg-elevated text-text-secondary hover:text-text-primary"
            }`}
          >
            Select
          </button>
        ))}
        <div className="w-px h-5 bg-border mx-1" />
        {PAD_SHAPES.map((ps) => (
          <button
            key={ps.value}
            onClick={() => { setTool(ps.value); setDrawingPoints([]); setSelectedPadIndex(null); setSelectedOutlineIndex(null); setSelectedCourtyardIndex(null); }}
            className={`px-2 py-1 text-xs rounded ${
              tool === ps.value ? "bg-copper text-white" : "bg-bg-elevated text-text-secondary hover:text-text-primary"
            }`}
          >
            {ps.label}
          </button>
        ))}
        <div className="w-px h-5 bg-border mx-1" />
        <button
          onClick={() => { setTool("draw-outline"); setDrawingPoints([]); }}
          className={`px-2 py-1 text-xs rounded ${
            tool === "draw-outline" ? "bg-copper text-white" : "bg-bg-elevated text-text-secondary hover:text-text-primary"
          }`}
        >
          Outline
        </button>
        <button
          onClick={() => { setTool("draw-courtyard"); setDrawingPoints([]); }}
          className={`px-2 py-1 text-xs rounded ${
            tool === "draw-courtyard" ? "bg-copper text-white" : "bg-bg-elevated text-text-secondary hover:text-text-primary"
          }`}
        >
          Courtyard
        </button>
        <div className="flex-1" />
        <span className="text-xs text-text-muted mr-2">
          {footprint.pads.length} pads
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
          onClick={() => onSave(footprint)}
          className="px-3 py-1 text-xs rounded bg-success text-white hover:opacity-90"
        >
          Save
        </button>
      </div>
      <div className="flex flex-1 overflow-hidden">
        <div className="flex-1 bg-bg-canvas" style={{ position: "relative" }}>
          <Stage
            ref={stageRef}
            width={width - (selectedPad ? 220 : 0)}
            height={height - 40}
            onMouseDown={handleMouseDown}
            onMouseUp={handleMouseUp}
            onDblClick={handleDblClick}
          >
            <Layer>
              {renderGrid()}
              {renderCourtyard()}
              {renderOutline()}
              {renderPads()}
              {renderDrawingPreview()}
            </Layer>
          </Stage>
        </div>
        <div className="w-56 border-l border-border bg-bg-surface flex flex-col overflow-y-auto">
          {selectedPad && selectedPadIndex !== null && (
            <div className="p-3 space-y-3">
              <h4 className="text-xs font-semibold text-text-primary uppercase tracking-wider">Pad Properties</h4>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Number</label>
                <input
                  type="text"
                  value={selectedPad.number}
                  onChange={(e) => updatePad(selectedPadIndex, { number: e.target.value })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                />
              </div>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Shape</label>
                <select
                  value={selectedPad.shape}
                  onChange={(e) => updatePad(selectedPadIndex, { shape: e.target.value as PadShape })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                >
                  <option value="Round">Round</option>
                  <option value="Rect">Rect</option>
                  <option value="Oval">Oval</option>
                  <option value="RoundRect">RoundRect</option>
                </select>
              </div>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Width</label>
                <input
                  type="number"
                  step={0.01}
                  value={selectedPad.size.width}
                  onChange={(e) => updatePad(selectedPadIndex, { size: { ...selectedPad.size, width: parseFloat(e.target.value) || 0 } })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                />
              </div>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Height</label>
                <input
                  type="number"
                  step={0.01}
                  value={selectedPad.size.height}
                  onChange={(e) => updatePad(selectedPadIndex, { size: { ...selectedPad.size, height: parseFloat(e.target.value) || 0 } })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                />
              </div>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Layer</label>
                <select
                  value={selectedPad.layer}
                  onChange={(e) => updatePad(selectedPadIndex, { layer: parseInt(e.target.value) })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                >
                  {Object.entries(LAYER_NAMES).map(([id, name]) => (
                    <option key={id} value={id}>{name}</option>
                  ))}
                </select>
              </div>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Drill {selectedPad.drill != null ? `(${selectedPad.drill}mm)` : "(none)"}</label>
                <input
                  type="number"
                  step={0.01}
                  value={selectedPad.drill ?? ""}
                  placeholder="No drill"
                  onChange={(e) => updatePad(selectedPadIndex, { drill: e.target.value ? parseFloat(e.target.value) : null })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                />
              </div>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Position X</label>
                <input
                  type="number"
                  step={0.01}
                  value={selectedPad.position.x}
                  onChange={(e) => updatePad(selectedPadIndex, { position: { ...selectedPad.position, x: parseFloat(e.target.value) || 0 } })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                />
              </div>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">Position Y</label>
                <input
                  type="number"
                  step={0.01}
                  value={selectedPad.position.y}
                  onChange={(e) => updatePad(selectedPadIndex, { position: { ...selectedPad.position, y: parseFloat(e.target.value) || 0 } })}
                  className="w-full bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary"
                />
              </div>

              <button
                onClick={() => {
                  updateFootprint((prev) => ({
                    ...prev,
                    pads: prev.pads.filter((_, i) => i !== selectedPadIndex),
                  }));
                  setSelectedPadIndex(null);
                }}
                className="w-full py-1.5 text-xs rounded bg-error/20 text-error border border-error/30 hover:bg-error/30 transition-colors"
              >
                Delete Pad
              </button>
            </div>
          )}

          {!selectedPad && (
            <div className="p-3 space-y-3">
              <h4 className="text-xs font-semibold text-text-primary uppercase tracking-wider">Actions</h4>

              <div className="space-y-1">
                <label className="block text-[10px] text-text-muted">3D Model</label>
                <div className="flex gap-1">
                  <input
                    type="text"
                    readOnly
                    value={modelPath || ""}
                    placeholder="None selected"
                    className="flex-1 bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary truncate"
                  />
                  <button
                    onClick={handleModelPick}
                    className="px-2 py-1 text-xs rounded bg-bg-elevated text-text-secondary hover:text-text-primary border border-border"
                  >
                    Browse
                  </button>
                </div>
              </div>

              <div className="pt-2 border-t border-border">
                <button
                  onClick={() => {
                    const json = JSON.stringify(footprint, null, 2);
                    const blob = new Blob([json], { type: "application/json" });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement("a");
                    a.href = url;
                    a.download = "footprint.json";
                    a.click();
                    URL.revokeObjectURL(url);
                  }}
                  className="w-full py-1.5 text-xs rounded bg-bg-elevated text-text-primary border border-border hover:bg-bg-surface transition-colors"
                >
                  Export as JSON
                </button>
              </div>

              <div className="text-[10px] text-text-muted space-y-1 pt-2 border-t border-border">
                <p><strong>Tools:</strong></p>
                <p>Click canvas to place pads/points</p>
                <p>Double-click to finish outline</p>
                <p>Drag pads to move them</p>
                <p>Delete key removes selected pad</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
