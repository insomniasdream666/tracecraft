import React from "react";
import { Line } from "react-konva";
import { Trace, LayerDef } from "../../types/pcb";

interface TraceRendererProps {
  trace: Trace;
  layers: LayerDef[];
  selected?: boolean;
}

function tracePointsEqual(a: { x: number; y: number }[], b: { x: number; y: number }[]) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i].x !== b[i].x || a[i].y !== b[i].y) return false;
  }
  return true;
}

export const TraceRenderer = React.memo(function TraceRenderer({ trace, layers, selected }: TraceRendererProps) {
  const layerColor = layers.find(l => l.id === trace.layer)?.color || "#D47A2F";
  const flat = trace.points.flatMap(p => [p.x, p.y]);
  return (
    <Line
      points={flat}
      stroke={selected ? "#5096E0" : layerColor}
      strokeWidth={trace.width * 4}
      lineCap="round"
      lineJoin="round"
    />
  );
}, (prev, next) => {
  if (prev.selected !== next.selected) return false;
  if (prev.trace.layer !== next.trace.layer) return false;
  if (prev.trace.width !== next.trace.width) return false;
  if (prev.trace.net_id !== next.trace.net_id) return false;
  if (!tracePointsEqual(prev.trace.points, next.trace.points)) return false;
  return true;
});
