import React, { useState, useCallback, useEffect } from "react";
import { Line, Circle } from "react-konva";
import { Point } from "../../lib/geometry";
import { useSchematicStore } from "../../stores/schematicStore";

export function WireTool() {
  const [points, setPoints] = useState<Point[]>([]);
  const [ghostPos, setGhostPos] = useState<Point | null>(null);
  const [isDrawing, setIsDrawing] = useState(false);
  const drawWire = useSchematicStore((s) => s.drawWire);

  const handleClick = useCallback(
    (worldPos: Point) => {
      if (!isDrawing) {
        setIsDrawing(true);
        setPoints([worldPos]);
      } else {
        const newPoints = [...points, worldPos];
        setPoints(newPoints);
      }
    },
    [isDrawing, points]
  );

  const handleMouseMove = useCallback(
    (worldPos: Point) => {
      if (isDrawing) setGhostPos(worldPos);
    },
    [isDrawing]
  );

  const handleDoubleClick = useCallback(() => {
    if (points.length >= 2) {
      drawWire(points);
    }
    setIsDrawing(false);
    setPoints([]);
    setGhostPos(null);
  }, [points, drawWire]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setIsDrawing(false);
        setPoints([]);
        setGhostPos(null);
      }
    },
    []
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  if (!isDrawing) return null;

  const flat = points.flatMap((p) => [p.x, p.y]);

  return (
    <>
      <Line points={flat} stroke="#D4783C" strokeWidth={2} lineCap="round" lineJoin="round" />
      {points.map((p, i) => (
        <Circle key={i} x={p.x} y={p.y} radius={2} fill="#D4783C" />
      ))}
      {ghostPos && points.length > 0 && (
        <Line
          points={[points[points.length - 1].x, points[points.length - 1].y, ghostPos.x, ghostPos.y]}
          stroke="#D4783C60"
          strokeWidth={1}
          dash={[4, 4]}
        />
      )}
    </>
  );
}

export { useCallback };
