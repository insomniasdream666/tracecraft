import { useCallback, useRef } from "react";
import { Point, snap } from "../lib/geometry";
import { GRID_SIZE, SNAP_DISTANCE } from "../lib/constants";

export interface CanvasInteractionHandlers {
  onMouseDown: (e: any) => void;
  onMouseMove: (e: any) => void;
  onMouseUp: (e: any) => void;
}

export function useCanvasInteraction() {
  const isDragging = useRef(false);
  const lastPos = useRef<Point>({ x: 0, y: 0 });

  const screenToWorld = useCallback(
    (stage: any, screenPos: Point): Point => {
      const stagePos = stage.getPosition();
      const scale = stage.scaleX();
      return {
        x: (screenPos.x - stagePos.x) / scale,
        y: (screenPos.y - stagePos.y) / scale,
      };
    },
    []
  );

  const snapToGrid = useCallback(
    (point: Point, gridSize: number = GRID_SIZE): Point => {
      return snap(point, gridSize);
    },
    []
  );

  return { screenToWorld, snapToGrid, isDragging, lastPos };
}
