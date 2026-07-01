import React from "react";
import { Line } from "react-konva";
import { Point } from "../../lib/geometry";

interface RatsnestLine {
  netName: string;
  start: Point;
  end: Point;
}

interface RatsnestRendererProps {
  lines: RatsnestLine[];
}

export function RatsnestRenderer({ lines }: RatsnestRendererProps) {
  return (
    <>
      {lines.map((line, i) => (
        <Line
          key={`ratsnest-${i}`}
          points={[line.start.x, line.start.y, line.end.x, line.end.y]}
          stroke="#F0A030"
          strokeWidth={0.5}
          dash={[2, 3]}
          opacity={0.4}
          listening={false}
        />
      ))}
    </>
  );
}
