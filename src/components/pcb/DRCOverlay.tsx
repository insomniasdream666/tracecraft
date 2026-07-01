import React from "react";
import { Circle, Text } from "react-konva";
import { Point } from "../../lib/geometry";

interface DRCViolation {
  severity: "error" | "warning";
  description: string;
  location: Point;
}

interface DRCOverlayProps {
  violations: DRCViolation[];
}

export function DRCOverlay({ violations }: DRCOverlayProps) {
  return (
    <>
      {violations.map((v, i) => (
        <Circle
          key={`drc-${i}`}
          x={v.location.x}
          y={v.location.y}
          radius={3}
          fill={v.severity === "error" ? "#E6454A" : "#F0A030"}
          opacity={0.8}
        />
      ))}
    </>
  );
}
