import React from "react";
import { Line, Group, Text } from "react-konva";
import { Zone } from "../../types/pcb";

interface ZoneRendererProps {
  zones: Record<number, Zone>;
  visibleLayers: Set<number>;
}

export function ZoneRenderer({ zones, visibleLayers }: ZoneRendererProps) {
  return (
    <>
      {Object.entries(zones).map(([id, zone]) => {
        if (!visibleLayers.has(zone.layer)) return null;
        const flat = zone.outline.flatMap(p => [p.x, p.y]);
        if (flat.length < 6) return null;
        return (
          <Group key={`zone-${id}`}>
            <Line
              points={flat}
              fill="#42BE6520"
              stroke="#42BE65"
              strokeWidth={1}
              closed
            />
            {zone.net_id !== null && (
              <Text
                text={`Zone net:${zone.net_id}`}
                x={zone.outline[0]?.x || 0}
                y={zone.outline[0]?.y || 0}
                fontSize={8}
                fill="#42BE65"
              />
            )}
          </Group>
        );
      })}
    </>
  );
}
