import React, { Component as ReactComponent } from "react";
import { Component } from "../../types/library";

class DetailErrorBoundary extends ReactComponent<
  { children: React.ReactNode },
  { hasError: boolean }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }
  static getDerivedStateFromError() {
    return { hasError: true };
  }
  render() {
    if (this.state.hasError) {
      return (
        <div className="p-3 text-center">
          <div className="text-xs text-error bg-red-900/20 border border-red-800 rounded px-2 py-1">
            Failed to render component detail
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

interface ComponentDetailProps {
  component: Component | null;
  loading?: boolean;
  error?: string | null;
  onPlaceSymbol: (id: string) => void;
  onPlaceFootprint: (id: string) => void;
  onEditFootprint?: (id: string) => void;
  onEditSymbol?: (component: Component) => void;
  onClose: () => void;
}

function computeBounds(
  items: { x: number; y: number }[]
): { minX: number; minY: number; width: number; height: number } | null {
  if (items.length === 0) return null;
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const p of items) {
    if (p.x < minX) minX = p.x;
    if (p.y < minY) minY = p.y;
    if (p.x > maxX) maxX = p.x;
    if (p.y > maxY) maxY = p.y;
  }
  return { minX, minY, width: maxX - minX, height: maxY - minY };
}

function SymbolPreview({ component }: { component: Component }) {
  const allPoints: { x: number; y: number }[] = [
    ...component.symbol.pins.flatMap((p) => {
      const len = p.length * (p.position.x < 0 ? -1 : 1);
      return [p.position, { x: p.position.x + len, y: p.position.y }];
    }),
    ...component.symbol.body.flatMap((g) => {
      if (g.type === "Rectangle") {
        return [
          { x: g.position.x - g.size.width / 2, y: g.position.y - g.size.height / 2 },
          { x: g.position.x + g.size.width / 2, y: g.position.y + g.size.height / 2 },
        ];
      }
      if (g.type === "Circle") {
        return [
          { x: g.center.x - g.radius, y: g.center.y - g.radius },
          { x: g.center.x + g.radius, y: g.center.y + g.radius },
        ];
      }
      if (g.type === "Line") {
        return [g.start, g.end];
      }
      if (g.type === "Text") {
        return [g.position];
      }
      return [];
    }),
  ];
  const bounds = computeBounds(allPoints);
  if (!bounds) {
    return <div className="text-[10px] text-text-muted italic">No symbol data</div>;
  }
  const pad = 2;
  const vx = bounds.minX - pad;
  const vy = bounds.minY - pad;
  const vw = bounds.width + pad * 2 || 10;
  const vh = bounds.height + pad * 2 || 10;

  return (
    <svg viewBox={`${vx} ${vy} ${vw} ${vh}`} className="w-full h-full" preserveAspectRatio="xMidYMid meet">
      {component.symbol.body.map((g, i) => {
        if (g.type === "Rectangle") {
          return (
            <rect
              key={i}
              x={g.position.x - g.size.width / 2}
              y={g.position.y - g.size.height / 2}
              width={g.size.width}
              height={g.size.height}
              fill={g.fill || "transparent"}
              stroke={g.stroke || "#E8E9F3"}
              strokeWidth={0.15}
            />
          );
        }
        if (g.type === "Circle") {
          return (
            <circle
              key={i}
              cx={g.center.x}
              cy={g.center.y}
              r={g.radius}
              fill={g.fill || "transparent"}
              stroke={g.stroke || "#E8E9F3"}
              strokeWidth={0.15}
            />
          );
        }
        if (g.type === "Line") {
          return (
            <line
              key={i}
              x1={g.start.x} y1={g.start.y}
              x2={g.end.x} y2={g.end.y}
              stroke={g.stroke || "#E8E9F3"}
              strokeWidth={g.width || 0.15}
            />
          );
        }
        return null;
      })}
      {component.symbol.pins.map((pin, i) => {
        const len = pin.length * (pin.position.x < 0 ? -1 : 1);
        return (
          <g key={`pin-${i}`}>
            <line
              x1={pin.position.x}
              y1={pin.position.y}
              x2={pin.position.x + len}
              y2={pin.position.y}
              stroke="#E8E9F3"
              strokeWidth={0.15}
            />
            <circle
              cx={pin.position.x + len}
              cy={pin.position.y}
              r={0.3}
              fill="#E8E9F3"
            />
            <text
              x={pin.position.x + len + (len < 0 ? -0.8 : 0.8)}
              y={pin.position.y + 0.4}
              fontSize={0.5}
              fill="#9496B8"
              textAnchor={len < 0 ? "end" : "start"}
            >
              {pin.name}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

function FootprintPreview({ component }: { component: Component }) {
  const fp = component.footprint;
  const allPoints: { x: number; y: number }[] = [
    ...fp.pads.flatMap((p) => [
      { x: p.position.x - p.size.width / 2, y: p.position.y - p.size.height / 2 },
      { x: p.position.x + p.size.width / 2, y: p.position.y + p.size.height / 2 },
    ]),
    ...fp.outline,
    ...fp.courtyard,
  ];
  const bounds = computeBounds(allPoints);
  if (!bounds) {
    return <div className="text-[10px] text-text-muted italic">No footprint data</div>;
  }
  const pad = 0.5;
  const vx = bounds.minX - pad;
  const vy = bounds.minY - pad;
  const vw = bounds.width + pad * 2 || 10;
  const vh = bounds.height + pad * 2 || 10;

  return (
    <svg viewBox={`${vx} ${vy} ${vw} ${vh}`} className="w-full h-full" preserveAspectRatio="xMidYMid meet">
      {fp.courtyard.length >= 2 && (
        <polygon
          points={fp.courtyard.map((p) => `${p.x},${p.y}`).join(" ")}
          fill="none"
          stroke="#5C5E7E"
          strokeWidth={0.06}
          strokeDasharray="0.2 0.2"
        />
      )}
      {fp.outline.length >= 2 && (
        <polygon
          points={fp.outline.map((p) => `${p.x},${p.y}`).join(" ")}
          fill="none"
          stroke="#42BE65"
          strokeWidth={0.1}
        />
      )}
      {fp.pads.map((pad, i) => {
        const x = pad.position.x - pad.size.width / 2;
        const y = pad.position.y - pad.size.height / 2;
        const radius = pad.shape === "Round" || pad.shape === "Oval"
          ? Math.min(pad.size.width, pad.size.height) / 2
          : pad.shape === "RoundRect" ? 0.3 : 0;
        return (
          <g key={`pad-${i}`}>
            <rect
              x={x}
              y={y}
              width={pad.size.width}
              height={pad.size.height}
              fill={pad.layer === 1 ? "#D47A2F" : "#A0793F"}
              stroke="#343760"
              strokeWidth={0.06}
              rx={radius}
              ry={pad.shape === "Oval" ? Math.min(pad.size.width, pad.size.height) / 2 : radius}
            />
            {pad.drill != null && (
              <circle
                cx={pad.position.x}
                cy={pad.position.y}
                r={pad.drill / 2}
                fill="#0F1020"
              />
            )}
            <text
              x={pad.position.x}
              y={pad.position.y - pad.size.height / 2 - 0.2}
              fontSize={0.3}
              fill="#E8E9F3"
              textAnchor="middle"
            >
              {pad.number}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

function ParameterTable({ parameters }: { parameters: Record<string, string> }) {
  const entries = Object.entries(parameters);
  if (entries.length === 0) {
    return <div className="text-[10px] text-text-muted italic py-2">No parameters</div>;
  }
  return (
    <table className="w-full text-[10px]">
      <thead>
        <tr className="text-text-muted border-b border-border-subtle">
          <th className="text-left py-0.5 font-medium">Parameter</th>
          <th className="text-right py-0.5 font-medium">Value</th>
        </tr>
      </thead>
      <tbody>
        {entries.map(([key, val]) => (
          <tr key={key} className="border-b border-border-subtle last:border-0">
            <td className="py-0.5 text-text-secondary">{key}</td>
            <td className="py-0.5 text-right text-text-primary">{val}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function ComponentDetail(props: ComponentDetailProps) {
  return (
    <DetailErrorBoundary>
      <ComponentDetailInner {...props} />
    </DetailErrorBoundary>
  );
}

function ComponentDetailInner({
  component,
  loading,
  error,
  onPlaceSymbol,
  onPlaceFootprint,
  onEditFootprint,
  onClose,
  onEditSymbol,
}: ComponentDetailProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center h-48 text-text-muted">
        <svg className="animate-spin h-5 w-5 mr-2" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
        Loading detail...
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-3 text-center">
        <div className="text-xs text-error bg-red-900/20 border border-red-800 rounded px-2 py-1">
          {error}
        </div>
      </div>
    );
  }

  if (!component) {
    return (
      <div className="flex items-center justify-center h-48 text-[10px] text-text-muted italic">
        Select a component to view details
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-surface">
        <div className="flex-1 min-w-0">
          <h3 className="text-sm font-semibold text-text-primary truncate">{component.name}</h3>
          {component.manufacturer && (
            <p className="text-[10px] text-text-muted truncate">{component.manufacturer}</p>
          )}
        </div>
        <button
          onClick={onClose}
          className="ml-2 text-text-muted hover:text-text-primary text-xs shrink-0"
        >
          Close
        </button>
      </div>

      <div className="flex-1 space-y-3 p-3">
        <div>
          <p className="text-[10px] text-text-secondary leading-relaxed">{component.description}</p>
          {component.category && (
            <span className="inline-block mt-1 px-1.5 py-0.5 text-[9px] rounded bg-bg-surface text-text-muted border border-border-subtle">
              {component.category}
            </span>
          )}
        </div>

        <div>
          <h4 className="text-[11px] font-medium text-text-primary mb-1">Parameters</h4>
          <div className="bg-bg-deep rounded border border-border-subtle px-2 py-1 max-h-32 overflow-y-auto">
            <ParameterTable parameters={component.parameters} />
          </div>
        </div>

        <div>
          <h4 className="text-[11px] font-medium text-text-primary mb-1">Symbol</h4>
          <div className="bg-bg-deep rounded border border-border-subtle flex items-center justify-center h-32 p-1">
            <SymbolPreview component={component} />
          </div>
        </div>

        <div>
          <h4 className="text-[11px] font-medium text-text-primary mb-1">Footprint</h4>
          <div className="bg-bg-deep rounded border border-border-subtle flex items-center justify-center h-32 p-1">
            <FootprintPreview component={component} />
          </div>
        </div>

        <div className="flex gap-2 pt-1">
          <button
            onClick={() => onPlaceSymbol(component.id)}
            className="flex-1 py-2 bg-copper text-white text-xs rounded hover:bg-copper-dark transition-colors"
          >
            Place in Schematic
          </button>
          <button
            onClick={() => onPlaceFootprint(component.id)}
            className="flex-1 py-2 bg-bg-surface text-text-primary text-xs rounded border border-border hover:bg-bg-elevated transition-colors"
          >
            Place in PCB
          </button>
          {onEditFootprint && (
            <button
              onClick={() => onEditFootprint(component.id)}
              className="flex-1 py-2 bg-bg-surface text-text-primary text-xs rounded border border-border hover:bg-bg-elevated transition-colors"
            >
              Edit Footprint
            </button>
          )}
        </div>
        {onEditSymbol && (
          <div className="pt-1">
            <button
              onClick={() => onEditSymbol(component)}
              className="w-full py-2 bg-copper/20 text-copper text-xs rounded border border-copper/40 hover:bg-copper/30 transition-colors"
            >
              Edit Symbol
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
