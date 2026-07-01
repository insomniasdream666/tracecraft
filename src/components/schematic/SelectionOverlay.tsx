import React, { useState, useCallback } from "react";
import { Rect, Line } from "react-konva";
import { Point } from "../../lib/geometry";

interface SelectionOverlayProps {
  onSelectionComplete?: (region: { x: number; y: number; width: number; height: number }) => void;
}

export function SelectionOverlay({ onSelectionComplete }: SelectionOverlayProps) {
  const [start, setStart] = useState<Point | null>(null);
  const [current, setCurrent] = useState<Point | null>(null);
  const [isSelecting, setIsSelecting] = useState(false);

  const handleMouseDown = useCallback(
    (worldPos: Point) => {
      setIsSelecting(true);
      setStart(worldPos);
      setCurrent(worldPos);
    },
    []
  );

  const handleMouseMove = useCallback(
    (worldPos: Point) => {
      if (isSelecting) setCurrent(worldPos);
    },
    [isSelecting]
  );

  const handleMouseUp = useCallback(() => {
    if (isSelecting && start && current) {
      const x = Math.min(start.x, current.x);
      const y = Math.min(start.y, current.y);
      const w = Math.abs(current.x - start.x);
      const h = Math.abs(current.y - start.y);
      onSelectionComplete?.({ x, y, width: w, height: h });
    }
    setIsSelecting(false);
    setStart(null);
    setCurrent(null);
  }, [isSelecting, start, current, onSelectionComplete]);

  if (!isSelecting || !start || !current) return null;

  const x = Math.min(start.x, current.x);
  const y = Math.min(start.y, current.y);
  const width = Math.abs(current.x - start.x);
  const height = Math.abs(current.y - start.y);

  return (
    <Rect
      x={x}
      y={y}
      width={width}
      height={height}
      fill="#5096E020"
      stroke="#5096E0"
      strokeWidth={1}
      dash={[4, 4]}
    />
  );
}
