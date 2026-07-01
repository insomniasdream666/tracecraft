import React from "react";
import { Line, Text } from "react-konva";
import { Bus } from "../../types/schematic";

interface BusRendererProps {
  bus: Bus;
}

export function BusRenderer({ bus }: BusRendererProps) {
  return (
    <Text
      text={`BUS: ${bus.name} [${bus.members.join(", ")}]`}
      x={0}
      y={0}
      fontSize={10}
      fill="#5096E0"
      fontStyle="bold"
    />
  );
}
