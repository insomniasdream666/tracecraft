import React from "react";
import { ToolButton } from "../common/ToolButton";
import { useSchematicStore } from "../../stores/schematicStore";
import { Eraser, Ruler, Type } from "lucide-react";

export function SchematicToolbar() {
  const clearSelection = useSchematicStore((s) => s.clearSelection);

  return (
    <div className="flex items-center gap-1 px-2 py-1 bg-bg-surface border-b border-border">
      <ToolButton icon={<Eraser size={14} />} label="Clear Selection" onClick={clearSelection} />
    </div>
  );
}
