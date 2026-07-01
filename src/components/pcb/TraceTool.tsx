import React, { useState, useCallback } from "react";
import { Line, Circle } from "react-konva";
import { Point } from "../../lib/geometry";
import { usePCBStore } from "../../stores/pcbStore";

interface TraceToolProps {
  onTraceComplete?: (points: Point[]) => void;
}

export function TraceTool({ onTraceComplete }: TraceToolProps) {
  const [points, setPoints] = useState<Point[]>([]);
  const [ghostPos, setGhostPos] = useState<Point | null>(null);
  const { activeLayer } = usePCBStore();
  const [isRouting, setIsRouting] = useState(false);

  const handleClick = useCallback(
    (worldPos: Point) => {
      if (!isRouting) {
        setIsRouting(true);
        setPoints([worldPos]);
      } else {
        const newPoints = [...points, worldPos];
        setPoints(newPoints);
      }
    },
    [isRouting, points]
  );

  const handleMouseMove = useCallback(
    (worldPos: Point) => {
      if (isRouting) {
        setGhostPos(worldPos);
      }
    },
    [isRouting]
  );

  const handleDoubleClick = useCallback(() => {
    if (points.length >= 2) {
      onTraceComplete?.(points);
    }
    setIsRouting(false);
    setPoints([]);
    setGhostPos(null);
  }, [points, onTraceComplete]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setIsRouting(false);
        setPoints([]);
        setGhostPos(null);
      }
    },
    []
  );

  React.useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const renderTraceInProgress = () => {
    if (points.length < 1) return null;
    const flat = points.flatMap((p) => [p.x, p.y]);
    return (
      <>
        <Line points={flat} stroke="#5096E0" strokeWidth={2} dash={[4, 4]} lineCap="round" />
        {points.map((p, i) => (
          <Circle key={i} x={p.x} y={p.y} radius={2} fill="#5096E0" />
        ))}
        {ghostPos && points.length > 0 && (
          <Line
            points={[points[points.length - 1].x, points[points.length - 1].y, ghostPos.x, ghostPos.y]}
            stroke="#5096E060"
            strokeWidth={1}
            dash={[2, 4]}
          />
        )}
      </>
    );
  };

  if (!isRouting) return null;

  return (
    <>
      {renderTraceInProgress()}
      <Circle
        x={ghostPos?.x ?? 0}
        y={ghostPos?.y ?? 0}
        radius={4}
        fill="#5096E040"
        listening={false}
      />
    </>
  );
}
