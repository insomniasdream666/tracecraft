import React from "react";
import { Layer, Line, Circle } from "react-konva";
import { invoke } from "@tauri-apps/api/core";
import { usePCBStore } from "../../stores/pcbStore";
import { useUIStore } from "../../stores/uiStore";
import { Point } from "../../lib/geometry";

interface DiffPairRoute {
  startP: Point;
  startN: Point;
}

export function DiffPairRouteTool() {
  const { board, activeLayer, loadBoard } = usePCBStore();
  const { panOffset, zoom } = useUIStore();
  const [route, setRoute] = React.useState<DiffPairRoute | null>(null);
  const [mousePos, setMousePos] = React.useState<Point | null>(null);

  if (!board) return null;

  const handleMouseDown = async (e: any) => {
    const pos = e.target.getStage()?.getPointerPosition();
    if (!pos) return;
    const worldPos: Point = {
      x: (pos.x - panOffset.x) / zoom,
      y: (pos.y - panOffset.y) / zoom,
    };
    if (!route) {
      setRoute({
        startP: { x: worldPos.x - 2.5, y: worldPos.y },
        startN: { x: worldPos.x + 2.5, y: worldPos.y },
      });
    } else {
      const pairIndex = 0;
      const layer = activeLayer;
      try {
        await invoke("route_diff_pair", {
          pairIndex,
          layer: { 0: layer },
          startP: route.startP, startN: route.startN,
          endP: worldPos, endN: worldPos,
        });
        loadBoard();
      } catch {}
      setRoute(null);
    }
  };

  const handleMouseMove = (e: any) => {
    const pos = e.target.getStage()?.getPointerPosition();
    if (!pos) return;
    setMousePos({
      x: (pos.x - panOffset.x) / zoom,
      y: (pos.y - panOffset.y) / zoom,
    });
  };

  const layerColor = board.layers.layers.find((l: any) => l.id === activeLayer)?.color || "#D47A2F";

  const previewP = route && mousePos ? [
    route.startP.x, route.startP.y,
    mousePos.x - 2.5, mousePos.y,
  ] : [];
  const previewN = route && mousePos ? [
    route.startN.x, route.startN.y,
    mousePos.x + 2.5, mousePos.y,
  ] : [];

  return (
    <Layer
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onDblClick={() => setRoute(null)}
    >
      {route && mousePos && (
        <>
          <Line points={previewP} stroke={layerColor} strokeWidth={0.2} dash={[2, 2]} />
          <Line points={previewN} stroke={layerColor} strokeWidth={0.2} dash={[2, 2]} />
          <Circle x={route.startP.x} y={route.startP.y} radius={0.3} fill="#FF4444" />
          <Circle x={route.startN.x} y={route.startN.y} radius={0.3} fill="#4444FF" />
        </>
      )}
    </Layer>
  );
}
