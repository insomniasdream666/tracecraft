import React from "react";
import { Eye, EyeOff } from "lucide-react";
import { usePCBStore } from "../../stores/pcbStore";

export function LayerPanel() {
  const { board, activeLayer, visibleLayers, setActiveLayer, toggleLayer } = usePCBStore();

  if (!board) return <div className="p-3 text-xs text-text-muted">No board loaded</div>;

  return (
    <div className="p-3">
      <h3 className="text-xs font-semibold text-text-primary mb-2">Layers</h3>
      <div className="space-y-1">
        {board.layers.layers.map((layer) => {
          const isVisible = visibleLayers.has(layer.id);
          const isActive = activeLayer === layer.id;
          return (
            <div
              key={layer.id}
              className={`flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer text-xs
                ${isActive ? "bg-bg-elevated ring-1 ring-info" : "hover:bg-bg-elevated"}`}
              onClick={() => setActiveLayer(layer.id)}
            >
              <button
                onClick={(e) => { e.stopPropagation(); toggleLayer(layer.id); }}
                className="text-text-secondary hover:text-text-primary"
              >
                {isVisible ? <Eye size={12} /> : <EyeOff size={12} />}
              </button>
              <div
                className="w-3 h-3 rounded"
                style={{ backgroundColor: layer.color }}
              />
              <span className="flex-1 text-text-secondary">{layer.name}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
