import React from "react";
import { Group, Rect, Circle, Line, Text } from "react-konva";
import { FootprintInstance } from "../../types/pcb";

interface PadRenderDef {
  number: string; shape: string; x: number; y: number; width: number; height: number; layer: number;
}

interface FootprintRendererProps {
  footprint: FootprintInstance;
  pads?: PadRenderDef[];
  selected?: boolean;
  onClick?: () => void;
}

function padsEqual(a: PadRenderDef[], b: PadRenderDef[]) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i].number !== b[i].number || a[i].x !== b[i].x || a[i].y !== b[i].y ||
        a[i].width !== b[i].width || a[i].height !== b[i].height || a[i].layer !== b[i].layer ||
        a[i].shape !== b[i].shape) return false;
  }
  return true;
}

export const FootprintRenderer = React.memo(function FootprintRenderer({ footprint, pads = [], selected, onClick }: FootprintRendererProps) {
  const layerColor = footprint.mirrored ? "#A0793F" : "#D47A2F";
  return (
    <Group
      x={footprint.position.x}
      y={footprint.position.y}
      onClick={onClick}
      onTap={onClick}
    >
      {pads.map((pad, i) => (
        <Rect
          key={i}
          x={pad.x - pad.width / 2}
          y={pad.y - pad.height / 2}
          width={pad.width}
          height={pad.height}
          fill={layerColor}
          stroke={selected ? "#5096E0" : "#5C5E7E"}
          strokeWidth={selected ? 2 : 0.5}
          cornerRadius={pad.shape === "Round" ? Math.min(pad.width, pad.height) / 2 : 0}
        />
      ))}
      <Rect
        x={-6}
        y={-4}
        width={12}
        height={8}
        stroke={selected ? "#5096E0" : "#5C5E7E"}
        strokeWidth={1}
        fill="transparent"
        cornerRadius={1}
      />
      <Text
        text={footprint.refdes}
        x={-5}
        y={-2}
        fontSize={7}
        fill="#E8E9F3"
      />
    </Group>
  );
}, (prev, next) => {
  if (prev.selected !== next.selected) return false;
  if (prev.onClick !== next.onClick) return false;
  if (prev.footprint.component_id !== next.footprint.component_id) return false;
  if (prev.footprint.position.x !== next.footprint.position.x) return false;
  if (prev.footprint.position.y !== next.footprint.position.y) return false;
  if (prev.footprint.rotation !== next.footprint.rotation) return false;
  if (prev.footprint.mirrored !== next.footprint.mirrored) return false;
  if (prev.footprint.refdes !== next.footprint.refdes) return false;
  if (prev.footprint.locked !== next.footprint.locked) return false;
  if (!padsEqual(prev.pads || [], next.pads || [])) return false;
  return true;
});
