import React from "react";
import { ToolButton } from "../common/ToolButton";
import { Route, Maximize2, Network, RefreshCw } from "lucide-react";

interface PcbToolbarProps {
  onAutoRoute: () => void;
  onZoomToFit: () => void;
  onSyncNetlist: () => void;
}

export function PcbToolbar({ onAutoRoute, onZoomToFit, onSyncNetlist }: PcbToolbarProps) {
  return (
    <div className="flex items-center gap-1">
      <ToolButton icon={<RefreshCw size={16} />} label="Sync Netlist" onClick={onSyncNetlist} />
      <ToolButton icon={<Route size={16} />} label="Auto-Route" onClick={onAutoRoute} />
      <ToolButton icon={<Network size={16} />} label="Refresh Ratsnest" onClick={onSyncNetlist} />
      <ToolButton icon={<Maximize2 size={16} />} label="Zoom to Fit" onClick={onZoomToFit} />
    </div>
  );
}
