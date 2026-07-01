import React from "react";
import { Group, Rect as KonvaRect, Circle, Line, Text } from "react-konva";
import { SymbolInstance } from "../../types/schematic";

interface SymbolRendererProps {
  symbol: SymbolInstance;
  selected?: boolean;
  onClick?: () => void;
}

export const SymbolRenderer = React.memo(function SymbolRenderer({ symbol, selected, onClick }: SymbolRendererProps) {
  return (
    <Group
      x={symbol.position.x}
      y={symbol.position.y}
      rotation={symbol.rotation ?? 0}
      onClick={onClick}
      onTap={onClick}
    >
      <KonvaRect
        width={24}
        height={18}
        x={-12}
        y={-9}
        fill="#2D2F52"
        stroke={selected ? "#5096E0" : "#5C5E7E"}
        strokeWidth={selected ? 2 : 1}
        cornerRadius={2}
      />
      <Text
        text={symbol.refdes || "?"}
        x={-8}
        y={-4}
        fontSize={9}
        fill="#E8E9F3"
      />
      <Text
        text={symbol.value || ""}
        x={-8}
        y={4}
        fontSize={6}
        fill="#9496B8"
      />
    </Group>
  );
}, (prev, next) => {
  if (prev.selected !== next.selected) return false;
  if (prev.symbol.component_id !== next.symbol.component_id) return false;
  if (prev.symbol.position.x !== next.symbol.position.x) return false;
  if (prev.symbol.position.y !== next.symbol.position.y) return false;
  if (prev.symbol.rotation !== next.symbol.rotation) return false;
  if (prev.symbol.refdes !== next.symbol.refdes) return false;
  if (prev.symbol.value !== next.symbol.value) return false;
  if (prev.symbol.mirrored !== next.symbol.mirrored) return false;
  return true;
});
