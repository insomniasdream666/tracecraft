import React from "react";

interface Property {
  label: string;
  value: string | number | boolean;
  onChange?: (value: string | number | boolean) => void;
  type?: "text" | "number" | "color" | "select" | "boolean";
  options?: { label: string; value: string }[];
}

interface PropertyPanelProps {
  title: string;
  properties: Property[];
}

function PropertyInput({ prop }: { prop: Property }) {
  if (!prop.onChange) {
    return <span className="flex-1 text-xs text-text-primary text-right">{String(prop.value)}</span>;
  }

  switch (prop.type) {
    case "boolean":
      return (
        <input
          type="checkbox"
          checked={!!prop.value}
          onChange={(e) => prop.onChange?.(e.target.checked)}
          className="rounded border-border bg-bg-deep text-copper focus:ring-copper"
        />
      );
    case "color":
      return (
        <input
          type="color"
          value={String(prop.value)}
          onChange={(e) => prop.onChange?.(e.target.value)}
          className="w-8 h-6 p-0 border border-border rounded cursor-pointer bg-bg-deep"
        />
      );
    case "select":
      return (
        <select
          value={String(prop.value)}
          onChange={(e) => prop.onChange?.(e.target.value)}
          className="flex-1 bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary focus:outline-none focus:border-info"
        >
          {prop.options?.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      );
    case "number":
      return (
        <input
          type="number"
          value={Number(prop.value)}
          onChange={(e) => prop.onChange?.(e.target.valueAsNumber)}
          className="flex-1 bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary focus:outline-none focus:border-info"
        />
      );
    default:
      return (
        <input
          type="text"
          value={String(prop.value)}
          onChange={(e) => prop.onChange?.(e.target.value)}
          className="flex-1 bg-bg-deep border border-border rounded px-2 py-1 text-xs text-text-primary focus:outline-none focus:border-info"
        />
      );
  }
}

export function PropertyPanel({ title, properties }: PropertyPanelProps) {
  return (
    <div className="p-3">
      <h3 className="text-sm font-semibold text-text-primary mb-3">{title}</h3>
      <div className="space-y-2">
        {properties.map((prop) => (
          <div key={prop.label} className="flex items-center justify-between gap-2">
            <label className="text-xs text-text-secondary w-24 shrink-0">{prop.label}</label>
            <div className={prop.type === "boolean" ? "" : "flex-1"}>
              <PropertyInput prop={prop} />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
