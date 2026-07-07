import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Square, Layers } from "lucide-react";
import toast from "react-hot-toast";

interface BoardProps {
  has_outline: boolean;
  width: number;
  height: number;
  layer_count: number;
  layer_names: string[];
}

export function BoardProperties() {
  const [props, setProps] = useState<BoardProps | null>(null);
  const [editW, setEditW] = useState(100);
  const [editH, setEditH] = useState(80);

  useEffect(() => {
    invoke<BoardProps>("get_board_properties").then((p) => {
      setProps(p);
      setEditW(p.width);
      setEditH(p.height);
    }).catch(() => {});
  }, []);

  const handleApplySize = async () => {
    try {
      await invoke("set_board_size", { width: editW, height: editH });
      const p = await invoke<BoardProps>("get_board_properties");
      setProps(p);
      toast.success(`Board size set to ${editW} × ${editH} mm`);
    } catch (e) {
      toast.error(String(e));
    }
  };

  if (!props) return null;

  return (
    <div className="p-3 space-y-4">
      <div className="flex items-center gap-2 text-xs font-semibold text-text-primary uppercase tracking-wider">
        <Square size={14} className="text-copper" />
        Board
      </div>

      <div className="space-y-2">
        <label className="text-xs text-text-secondary">Width (mm)</label>
        <input
          type="number"
          value={editW}
          onChange={(e) => setEditW(Number(e.target.value))}
          min={1}
          step={1}
          className="w-full px-2 py-1.5 rounded bg-bg-elevated border border-border text-text-primary text-sm outline-none focus:border-copper transition-colors"
        />
      </div>

      <div className="space-y-2">
        <label className="text-xs text-text-secondary">Height (mm)</label>
        <input
          type="number"
          value={editH}
          onChange={(e) => setEditH(Number(e.target.value))}
          min={1}
          step={1}
          className="w-full px-2 py-1.5 rounded bg-bg-elevated border border-border text-text-primary text-sm outline-none focus:border-copper transition-colors"
        />
      </div>

      <button
        onClick={handleApplySize}
        className="w-full px-3 py-1.5 rounded bg-copper text-white text-xs font-medium hover:bg-copper-dark transition-colors"
      >
        Apply Board Size
      </button>

      <div className="flex items-center gap-2 text-xs font-semibold text-text-primary uppercase tracking-wider pt-2 border-t border-border">
        <Layers size={14} className="text-copper" />
        Layers ({props.layer_count})
      </div>

      <div className="space-y-1">
        {props.layer_names.map((name, i) => (
          <div key={i} className="flex items-center gap-2 px-2 py-1 rounded bg-bg-elevated text-xs text-text-secondary">
            <div className="w-2 h-2 rounded-full bg-copper/60" />
            {name}
          </div>
        ))}
      </div>
    </div>
  );
}
